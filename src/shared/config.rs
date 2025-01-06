use std::env;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
  pub master_key: String,
  pub jwt_secret: String,
}

impl Default for Config {
  fn default() -> Self {
    let master_key =
      env::var("MASTER_KEY").unwrap_or_else(|_| "DEV_MASTER_KEY".to_string());
    let jwt_secret =
      env::var("JWT_SECRET").unwrap_or_else(|_| "DEV_JWT_SECRET".to_string());
    Self {
      master_key,
      jwt_secret,
    }
  }
}
