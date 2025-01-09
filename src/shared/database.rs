use std::sync::Arc;

use super::config::Config;

pub struct Database {
  pub dynamo_client: Arc<aws_sdk_dynamodb::Client>,
}

impl Database {
  pub fn new(config: &Config) -> Self {
    let client = aws_sdk_dynamodb::Client::new(
      config
        .aws_config
        .as_ref()
        .expect("Database must be initialized with AWS SDK"),
    );
    Self {
      dynamo_client: Arc::new(client),
    }
  }
}
