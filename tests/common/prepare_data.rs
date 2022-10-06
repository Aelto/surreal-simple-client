use serde_json::Value;
use surreal_simple_client::rpc::RpcResult;
use surreal_simple_client::SurrealClient;
use surreal_simple_querybuilder::querybuilder::QueryBuilder;

use super::models;

pub async fn prepare_data(client: &mut SurrealClient) -> RpcResult<()> {
  use models::file_schema::schema::model as file;
  use models::user_schema::schema::model as user;

  // delete all User nodes from the database
  client
    .send_query(QueryBuilder::new().delete(user).build(), Value::Null)
    .await?
    .await?;

  // delete all File nodes from the database
  client
    .send_query(QueryBuilder::new().delete(file).build(), Value::Null)
    .await?
    .await?;

  Ok(())
}
