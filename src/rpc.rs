use std::fmt::Display;

use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite;
use thiserror::Error;

use crate::errors::SurrealError;

pub type RpcResult<T> = Result<T, RpcChannelError>;

#[derive(Debug, Error)]
pub enum RpcChannelError {
  SurrealBodyParsingError { inner: serde_json::Error },
  SocketError { inner: tungstenite::Error },
  SurrealQueryError { inner: SurrealError },
  OneshotError { inner: oneshot::error::RecvError },
}

impl From<tungstenite::Error> for RpcChannelError {
  fn from(inner: tungstenite::Error) -> Self {
    Self::SocketError { inner }
  }
}

impl From<serde_json::Error> for RpcChannelError {
  fn from(inner: serde_json::Error) -> Self {
    Self::SurrealBodyParsingError { inner }
  }
}

impl From<oneshot::error::RecvError> for RpcChannelError {
  fn from(inner: oneshot::error::RecvError) -> Self {
    Self::OneshotError { inner }
  }
}

impl Display for RpcChannelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RpcChannelError::SurrealBodyParsingError { inner } => {
        write!(f, "Surreal body parsing errror: {}", inner)
      }
      RpcChannelError::SocketError { inner } => write!(f, "RPC socket error: {}", inner),
      RpcChannelError::SurrealQueryError { inner } => {
        write!(f, "Surreal query errror: {:?}", inner)
      }
      RpcChannelError::OneshotError { inner } => {
        write!(f, "Oneshot receiver error: {inner}")
      }
    }
  }
}

#[cfg(feature = "actix")]
impl actix_web::ResponseError for RpcChannelError {
  fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
    actix_web::HttpResponse::build(self.status_code())
      .insert_header(actix_web::http::header::ContentType::html())
      .body(match self {
        RpcChannelError::SurrealBodyParsingError { inner: _ } => {
          "Failed to parse results from the database"
        }
        RpcChannelError::SocketError { inner: _ } => "RPC socket failure",
        RpcChannelError::SurrealQueryError { inner: _ } => {
          "Incorrect query was sent to the database"
        }
        RpcChannelError::OneshotError { inner: _ } => "SPSC channel failure",
      })
  }
}
