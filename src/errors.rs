use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealError {
  id: String,
  error: SurrealInternalError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealInternalError {
  code: i32,
  message: String,
}
