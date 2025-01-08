use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginRTO {
  pub access_token: String,
  pub refresh_token: String,
}
