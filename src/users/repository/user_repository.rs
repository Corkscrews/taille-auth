use aws_sdk_dynamodb::{
  error::SdkError,
  operation::{get_item::GetItemError, put_item::PutItemError},
  types::AttributeValue,
};
use mongodb::bson::{doc, to_document};
use thiserror::Error;

use crate::{
  shared::database::{
    Database, DynamoDatabase, InMemoryDatabase, MongoDatabase,
  },
  users::model::user::User,
};

#[derive(Debug, Error)]
pub enum UserRepositoryError {
  #[error("Serialization error: {0}")]
  SerializationError(#[from] serde_dynamo::Error),

  #[error("Get item error: {0}")]
  GetItemError(#[from] SdkError<GetItemError>),

  #[error("Put item error: {0}")]
  PutItemError(#[from] SdkError<PutItemError>),

  #[error("Other error: {0}")]
  Other(String),
}

pub enum FindOneProperty<'a> {
  Uuid(&'a str),
  Email(&'a str),
}

impl FindOneProperty<'_> {
  fn to_key_value(&self) -> (&str, AttributeValue) {
    match self {
      FindOneProperty::Uuid(uuid) => {
        ("uuid", AttributeValue::S(uuid.to_string()))
      }
      FindOneProperty::Email(email) => {
        ("email", AttributeValue::S(email.to_string()))
      }
    }
  }
  fn to_mongo_key_value(&self) -> mongodb::bson::Document {
    match self {
      FindOneProperty::Uuid(uuid) => {
        doc! { "uuid": uuid }
      }
      FindOneProperty::Email(email) => {
        doc! { "email": email }
      }
    }
  }
}

pub trait UserRepository {
  async fn find_one(
    &self,
    property: FindOneProperty,
  ) -> Result<User, UserRepositoryError>;
  async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError>;
  async fn create(&self, user: User) -> Result<(), UserRepositoryError>;
}

pub struct UserRepositoryImpl<DB: Database> {
  database: DB,
}

impl<DB: Database> UserRepositoryImpl<DB> {
  pub fn new(database: DB) -> Self {
    Self { database }
  }
}

impl UserRepository for UserRepositoryImpl<DynamoDatabase> {
  async fn find_one<'a>(
    &self,
    property: FindOneProperty<'a>,
  ) -> Result<User, UserRepositoryError> {
    let (key, value) = property.to_key_value();
    let result = self
      .database
      .client
      .get_item()
      .table_name("users")
      .key(key, value)
      .send()
      .await?;
    if let Some(item) = result.item {
      let user: User = serde_dynamo::from_item(item).unwrap();
      return Ok(user);
    }
    Err(UserRepositoryError::Other(String::from("No item")))
  }

  async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError> {
    Ok(Vec::new())
  }

  async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
    let item = serde_dynamo::to_item(&user).unwrap();
    self
      .database
      .client
      .put_item()
      .table_name("users")
      .set_item(Some(item))
      .send()
      .await?;
    Ok(())
  }
}

// ### MongoDB implementation ###

impl UserRepository for UserRepositoryImpl<MongoDatabase> {
  async fn find_one<'a>(
    &self,
    property: FindOneProperty<'a>,
  ) -> Result<User, UserRepositoryError> {
    let result: Option<User> = self
      .database
      .client
      .database("test")
      .collection("users")
      .find_one(property.to_mongo_key_value())
      .await
      .unwrap(); // TODO: Remove unwrap
    if let Some(user) = result {
      return Ok(user);
    }
    Err(UserRepositoryError::Other(String::from("No item")))
  }

  async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError> {
    Ok(Vec::new())
  }

  async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
    _ = self
      .database
      .client
      .database("test")
      .collection("users")
      .insert_one(to_document(&user).unwrap())
      .await;
    Ok(())
  }
}

impl UserRepository for UserRepositoryImpl<InMemoryDatabase> {
  async fn find_one<'a>(
    &self,
    property: FindOneProperty<'a>,
  ) -> Result<User, UserRepositoryError> {
    // Acquire read lock
    self
      .database
      .users
      .read()
      .unwrap()
      .iter()
      .find(|user| match property {
        FindOneProperty::Uuid(uuid) => user.uuid == uuid,
        FindOneProperty::Email(email) => user.email == email,
      })
      .cloned()
      .ok_or(UserRepositoryError::Other(String::new()))
  }

  async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
    let mut users = self.database.users.write().unwrap(); // Acquire write lock
    users.push(user.clone());
    Ok(())
  }

  async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError> {
    Ok(self.database.users.read().unwrap().clone())
  }
}
