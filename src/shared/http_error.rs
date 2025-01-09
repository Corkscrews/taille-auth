use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpError {
  pub message: String,
}

impl From<&str> for HttpError {
  fn from(message: &str) -> Self {
    Self {
      message: String::from(message),
    }
  }
}
