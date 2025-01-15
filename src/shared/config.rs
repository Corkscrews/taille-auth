use std::env;

#[derive(Clone, Debug)]
pub struct Config {
  pub host: String,
  pub master_key: String,
  pub jwt_secret: String,
}

impl Config {
  pub async fn default() -> Self {
    let host =
      env::var("host").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let master_key =
      env::var("MASTER_KEY").unwrap_or_else(|_| "DEV_MASTER_KEY".to_string());
    let jwt_secret =
      env::var("JWT_SECRET").unwrap_or_else(|_| "DEV_JWT_SECRET".to_string());
    Self {
      host,
      master_key,
      jwt_secret
    }
  }
}
