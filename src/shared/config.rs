use std::env;

#[derive(Clone, Debug)]
pub struct Config {
  pub server_address: String,
  pub master_key: String,
  pub jwt_secret: String,
}

impl Config {
  pub async fn default() -> Self {
    let server_address =
      env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let master_key =
      env::var("MASTER_KEY").unwrap_or_else(|_| "DEV_MASTER_KEY".to_string());
    let jwt_secret =
      env::var("JWT_SECRET").unwrap_or_else(|_| "DEV_JWT_SECRET".to_string());
    Self {
      server_address,
      master_key,
      jwt_secret
    }
  }
}
