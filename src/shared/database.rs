use std::{
  env,
  sync::{Arc, RwLock},
};

use crate::users::model::user::User;

use super::config::Config;

pub trait Database: Sized {
  async fn new(config: &Config) -> Option<Self>;
}

pub struct DynamoDatabase {
  pub client: Arc<aws_sdk_dynamodb::Client>,
}

impl Database for DynamoDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&aws_config);
    Some(Self {
      client: Arc::new(client),
    })
  }
}

use mongodb::Client;

pub struct MongoDatabase {
  pub client: mongodb::Client,
}

impl Database for MongoDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    if let Ok(mongo_url) = env::var("MONGO_URL") {
      println!("Starting MongoDB client at {}", mongo_url);
      return Some(Self {
        // Create a new MongoDB client with the parsed options
        client: Client::with_uri_str(mongo_url).await.unwrap(),
      });
    }
    None
  }
}

pub struct InMemoryDatabase {
  pub users: Arc<RwLock<Vec<User>>>,
}

impl Database for InMemoryDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    Some(Self {
      users: Arc::new(RwLock::new(Vec::new())),
    })
  }
}
