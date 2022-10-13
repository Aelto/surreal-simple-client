use nanoid::nanoid;
use surreal_simple_client::rpc::RpcResult;
use surreal_simple_client::SurrealClient;

use super::prepare_data;

pub async fn open_connection() -> RpcResult<SurrealClient> {
  let mut client = SurrealClient::new("ws://127.0.0.1:8000/rpc")
    .await
    .expect("RPC handshake error");

  client.signin("root", "root").await.expect("Signin error");
  client
    .use_namespace(nanoid!(), nanoid!())
    .await
    .expect("Namespace error");

  // every time we open a new connection for the test we prepare the data, flush
  // everything we may not want and add additional data we may need.
  prepare_data(&mut client).await?;

  Ok(client)
}
