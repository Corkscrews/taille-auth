use std::{env, sync::Arc};

use super::config::Config;

pub struct DynamoDatabase {
  pub dynamo_client: Arc<aws_sdk_dynamodb::Client>,
}

impl DynamoDatabase {
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

use mongodb::Client;

pub struct MongoDatabase {
  pub mongo_client: mongodb::Client,
}

impl MongoDatabase {
  pub async fn new(config: &Config) -> Self {
    let mongo_url =
      env::var("MONGO_URL").expect("MONGO_URL environment variable not set");
    println!("Starting MongoDB client at {}", mongo_url);
    Self {
      // Create a new MongoDB client with the parsed options
      mongo_client: Client::with_uri_str(mongo_url).await.unwrap(),
    }
  }
}
