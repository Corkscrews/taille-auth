use super::config::Config;

pub trait Database: Sized {
  async fn new(config: &Config) -> Option<Self>;
  async fn stats(&self) -> DatabaseStats;
}

pub struct DatabaseStats {
  pub connected: bool,
  pub name: String,
}

#[cfg(all(feature = "dynamodb", not(test)))]
pub async fn resolve_database(
  config: &Config,
) -> crate::shared::database::DynamoDatabase {
  crate::shared::database::DynamoDatabase::new(&config)
    .await
    .unwrap()
}

#[cfg(all(feature = "mongodb", not(test)))]
pub async fn resolve_database(
  config: &Config,
) -> crate::shared::database::MongoDatabase {
  crate::shared::database::MongoDatabase::new(config)
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

#[cfg(all(feature = "dynamodb", not(test)))]
pub struct DynamoDatabase {
  pub client: std::sync::Arc<aws_sdk_dynamodb::Client>,
}

#[cfg(all(feature = "dynamodb", not(test)))]
impl Database for DynamoDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&aws_config);
    Some(Self {
      client: std::sync::Arc::new(client),
    })
  }
  async fn stats(&self) -> DatabaseStats {
    let result = self
      .client
      .database("admin")
      .run_command(mongodb::bson::doc! { "ping": 1 })
      .await;
    DatabaseStats {
      connected: result.is_ok(),
      name: String::from("MongoDB"),
    }
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
  async fn stats(&self) -> DatabaseStats {
    let result = self
      .client
      .database("admin")
      .run_command(mongodb::bson::doc! { "ping": 1 })
      .await;
    DatabaseStats {
      connected: result.is_ok(),
      name: String::from("MongoDB"),
    }
  }
}

#[cfg(any(feature = "in-memory", test))]
pub struct InMemoryDatabase {
  pub users:
    std::sync::Arc<std::sync::RwLock<Vec<crate::users::model::user::User>>>,
}

#[cfg(any(feature = "in-memory", test))]
impl Database for InMemoryDatabase {
  async fn new(_config: &Config) -> Option<Self> {
    Some(Self {
      users: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
    })
  }
  async fn stats(&self) -> DatabaseStats {
    DatabaseStats {
      connected: true,
      name: String::from("In-Memory"),
    }
  }
}
