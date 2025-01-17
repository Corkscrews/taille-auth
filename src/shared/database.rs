use std::sync::{Arc, RwLock};

use crate::users::model::user::User;

use super::config::Config;

pub trait Database: Sized {
  async fn new(config: &Config) -> Option<Self>;
}

#[cfg(feature = "dynamodb")]
pub async fn resolve_database(
  config: &Config,
) -> crate::shared::database::DynamoDatabase {
  crate::shared::database::DynamoDatabase::new(&config)
    .await
    .unwrap()
}

#[cfg(feature = "mongodb")]
pub async fn resolve_database(
  config: &Config,
) -> crate::shared::database::MongoDatabase {
  crate::shared::database::MongoDatabase::new(&config)
    .await
    .unwrap()
}

#[cfg(any(feature = "in-memory", test))]
pub async fn resolve_database(
  config: &Config,
) -> crate::shared::database::InMemoryDatabase {
  crate::shared::database::InMemoryDatabase::new(config)
    .await
    .unwrap()
}

#[cfg(feature = "dynamodb")]
pub struct DynamoDatabase {
  pub client: Arc<aws_sdk_dynamodb::Client>,
}

#[cfg(feature = "dynamodb")]
impl Database for DynamoDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&aws_config);
    Some(Self {
      client: Arc::new(client),
    })
  }
}

#[cfg(feature = "mongodb")]
pub struct MongoDatabase {
  pub client: mongodb::Client,
}

#[cfg(feature = "mongodb")]
impl Database for MongoDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    if let Ok(mongo_url) = std::env::var("MONGO_URL") {
      println!("Starting MongoDB client at {}", mongo_url);
      return Some(Self {
        // Create a new MongoDB client with the parsed options
        client: mongodb::Client::with_uri_str(mongo_url).await.unwrap(),
      });
    }
    None
  }
}

#[cfg(any(feature = "in-memory", test))]
pub struct InMemoryDatabase {
  pub users: Arc<RwLock<Vec<User>>>,
}

#[cfg(any(feature = "in-memory", test))]
impl Database for InMemoryDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    Some(Self {
      users: Arc::new(RwLock::new(Vec::new())),
    })
  }
}
