use std::env;

use aws_config::SdkConfig;

#[derive(Clone, Debug)]
pub struct Config {
  pub master_key: String,
  pub jwt_secret: String,
  pub aws_config: Option<SdkConfig>,
}

impl Config {
  pub async fn default() -> Self {
    let master_key =
      env::var("MASTER_KEY").unwrap_or_else(|_| "DEV_MASTER_KEY".to_string());
    let jwt_secret =
      env::var("JWT_SECRET").unwrap_or_else(|_| "DEV_JWT_SECRET".to_string());
    Self {
      master_key,
      jwt_secret,
      aws_config: Some(aws_config::load_from_env().await),
    }
  }
}
