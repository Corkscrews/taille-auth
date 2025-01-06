use std::sync::Arc;

pub struct Database {
  pub dynamo_client: Arc<aws_sdk_dynamodb::Client>,
}

impl Database {
  pub async fn new() -> Self {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    Self {
      dynamo_client: Arc::new(client),
    }
  }
}
