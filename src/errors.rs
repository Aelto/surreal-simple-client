use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite;

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

#[derive(Debug)]
pub enum Error {
  Tungstenite(tungstenite::Error),
  Json(serde_json::Error),
  Oneshot(oneshot::error::RecvError),
}

impl From<tungstenite::Error> for Error {
  fn from(e: tungstenite::Error) -> Self {
    Error::Tungstenite(e)
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Error::Json(e)
  }
}

impl From<oneshot::error::RecvError> for Error {
  fn from(e: oneshot::error::RecvError) -> Self {
    Error::Oneshot(e)
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Tungstenite(e) => e.fmt(f),
      Error::Json(e) => e.fmt(f),
      Error::Oneshot(e) => e.fmt(f),
    }
  }
}

impl std::error::Error for Error {}
