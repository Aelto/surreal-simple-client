use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::Future;
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
use crate::SurrealResponseData;

type SurrealResponseSender = oneshot::Sender<SurrealResponseData>;

#[derive(Debug)]
pub struct SurrealResponse {
  receiver: oneshot::Receiver<SurrealResponseData>,
}
impl Future for SurrealResponse {
  type Output = <oneshot::Receiver<SurrealResponseData> as Future>::Output;

  fn poll(
    self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    // SAFETY
    // As long as nothing ever hands out an `&(mut) Receiver` this is safe.
    unsafe {
      self
        .map_unchecked_mut(|response| &mut response.receiver)
        .poll(cx)
    }
  }
}

pub struct SurrealClient {
  socket_sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
  resp_sink: mpsc::UnboundedSender<(String, SurrealResponseSender)>,
}

impl SurrealClient {
  pub async fn new(url: &str) -> RpcResult<Self> {
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
                match serde_json::from_str::<SurrealResponseData>(&json_message) {
                  Ok(response) => if let Some(sender) = requests.remove(&response.id) {
                    if let Err(_) = sender.send(response) {
                      // do nothing at the moment, an error from a .send() call
                      // means the receiver is no longer listening. Which is a
                      // possible & valid state.
                    }
                  },
                  Err(_) => {
                    // TODO: this error should be handled, probably by sending
                    // it through the `sender`. But that would require the
                    // `SurrealResponseSender` to accept an enum as it only accepts
                    // valid data at the moment.
                  },
                };
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
      .await?
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
      .await?
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

    Ok(SurrealResponse { receiver: rx })
  }

  /// Send a query using the current socket connection then return the raw [SurrealResponse]
  pub async fn send_query(&mut self, query: String, params: Value) -> RpcResult<SurrealResponse> {
    Ok(self.send_message("query", json!([query, params])).await?)
  }

  /// Send a query using the current socket connection then return the **first** [Value]
  /// from the received [SurrealResponse]
  ///
  /// Use [`Self::find_one()`] instead to get a typed return value.
  async fn find_one_value(&mut self, query: String, params: Value) -> RpcResult<Option<Value>> {
    let response = self.send_query(query, params).await?.await?;

    Ok(
      response
        .get_nth_query_result(0)
        .and_then(|query_results| query_results.results().first().cloned()),
    )
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

  /// Fetch the value for the given `key` out of the first row that is returned by
  /// the supplied `query`. If the key is missing then [None] is returned.
  pub async fn find_one_key<T: DeserializeOwned>(
    &mut self, key: &str, query: String, params: Value,
  ) -> RpcResult<Option<T>> {
    let response = self.send_query(query, params).await?.await?;

    let value = response
      .get_nth_query_result(0)
      .and_then(|query_results| query_results.results_key(key).first().cloned().cloned());

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
    let response = self.send_query(query, params).await?.await?;

    Ok(
      response
        .get_nth_query_result(0)
        .and_then(|query_results| Some(query_results.results().clone()))
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

  /// Get the value for every row that were returned by the supplied `query` and
  /// where `key` exists. If the `key` is missing from a row then the row will
  /// be filtered out of the returned [Vec].
  pub async fn find_many_key<T: DeserializeOwned>(
    &mut self, key: &str, query: String, params: Value,
  ) -> RpcResult<Vec<T>> {
    let response = self.send_query(query, params).await?.await?;

    let values = response
      .get_nth_query_result(0)
      .and_then(|query_results| Some(query_results.results_key(key)))
      .unwrap_or_default();

    let deser_result: Vec<T> =
      serde_json::from_value(Value::Array(values.into_iter().cloned().collect()))?;

    Ok(deser_result)
  }
}
