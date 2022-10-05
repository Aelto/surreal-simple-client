use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::SinkExt;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde_json::json;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

use crate::rpc::RpcResult;
use crate::SurrealMessage;
use crate::SurrealResponse;

type SurrealResponseSender = oneshot::Sender<SurrealResponse>;

pub struct SurrealClient {
  socket_sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
  resp_sink: mpsc::UnboundedSender<(String, SurrealResponseSender)>,
}

impl SurrealClient {
  pub async fn new(url: &str) -> tokio_tungstenite::tungstenite::Result<Self> {
    let (socket, _) = tokio_tungstenite::connect_async(url).await?;
    let (socket_sink, mut socket_stream) = socket.split();
    let (resp_sink, resp_stream) = tokio::sync::mpsc::unbounded_channel();
    let mut recv_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(resp_stream);

    tokio::spawn(async move {
      let mut requests: HashMap<String, SurrealResponseSender> = HashMap::new();

      loop {
        tokio::select! {
          receiver = recv_stream.next() => {
            if let Some((id, sender)) = receiver {
              requests.insert(id, sender);
            }
          },

          res = socket_stream.next() => {
            if let Some(Ok(Message::Text(json_message))) = res {
              if let Ok(response) = serde_json::from_str::<SurrealResponse>(&json_message) {
                if let Some(sender) = requests.remove(&response.id) {
                  sender.send(response).unwrap();
                }
              }
            }
          },
        }
      }
    });

    Ok(Self {
      socket_sink,
      resp_sink,
    })
  }

  pub async fn signin<T: AsRef<str>>(&mut self, user: T, pass: T) -> RpcResult<()>
  where
    String: From<T>,
  {
    self
      .send_message(
        "signin",
        json!([{
          "user": String::from(user),
          "pass": String::from(pass)
        }]),
      )
      .await?;

    Ok(())
  }

  pub async fn use_namespace<T: AsRef<str>>(&mut self, namespace: T, database: T) -> RpcResult<()>
  where
    String: From<T>,
  {
    self
      .send_message(
        "use",
        json!([String::from(namespace), String::from(database)]),
      )
      .await?;

    Ok(())
  }

  pub async fn send_message(
    &mut self, method: &'static str, params: Value,
  ) -> RpcResult<SurrealResponse> {
    const ALPHABET: [char; 36] = [
      '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
      'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    let message = SurrealMessage {
      id: nanoid::nanoid!(10, &ALPHABET),
      method: method.to_owned(),
      params,
    };

    let (tx, rx) = oneshot::channel();

    self.resp_sink.send((message.id.clone(), tx)).unwrap();
    self
      .socket_sink
      .send(Message::Text(serde_json::to_string(&message).unwrap()))
      .await?;

    let response = rx.await?;

    Ok(response)
  }

  /// Send a query using the current socket connection then return the raw [SurrealResponse]
  pub async fn send_query(&mut self, query: String, params: Value) -> RpcResult<SurrealResponse> {
    self.send_message("query", json!([query, params])).await
  }

  /// Send a query using the current socket connection then return the **first** [Value]
  /// from the received [SurrealResponse]
  ///
  /// Use [`Self::find_one()`] instead to get a typed return value.
  async fn find_one_value(&mut self, query: String, params: Value) -> RpcResult<Option<Value>> {
    let response = self.send_query(query, params).await?;

    match response.get_results() {
      Some(array) => match array.first() {
        Some(first_object) => Ok(Some(first_object.to_owned())),
        None => Ok(None),
      },
      None => Ok(None),
    }
  }

  /// Send a query using the current socket connection then return the **first** [T]
  /// from the received [SurrealResponse].
  pub async fn find_one<T: DeserializeOwned>(
    &mut self, query: String, params: Value,
  ) -> RpcResult<Option<T>> {
    let value = self.find_one_value(query, params).await?;

    match value {
      None => Ok(None),
      Some(inner) => {
        let deser_result = serde_json::from_value::<T>(inner)?;

        Ok(Some(deser_result))
      }
    }
  }

  /// Send a query using the current socket connection then return the [Value]s
  /// from the received [SurrealResponse]
  ///
  /// Use [`Self::find_many()`] instead to get a typed return value.
  async fn find_many_values(&mut self, query: String, params: Value) -> RpcResult<Vec<Value>> {
    let response = self.send_query(query, params).await?;

    Ok(
      response
        .get_results()
        .and_then(|array| Some(array.to_owned()))
        .unwrap_or_default(),
    )
  }

  /// Send a query using the current socket connection then return the many [`<T>`]
  /// from the received [SurrealResponse].
  pub async fn find_many<T: DeserializeOwned>(
    &mut self, query: String, params: Value,
  ) -> RpcResult<Vec<T>> {
    let values = self.find_many_values(query, params).await?;
    let deser_result: Vec<T> = serde_json::from_value(Value::Array(values))?;

    Ok(deser_result)
  }
}