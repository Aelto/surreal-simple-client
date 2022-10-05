use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealMessage {
  pub id: String,
  pub method: String,
  pub params: Value,
}
