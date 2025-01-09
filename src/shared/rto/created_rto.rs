use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedRto {
  pub uuid: String,
}

impl From<&str> for CreatedRto {
  fn from(uuid: &str) -> Self {
    Self {
      uuid: String::from(uuid),
    }
  }
}
