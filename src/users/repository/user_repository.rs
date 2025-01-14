use aws_sdk_dynamodb::{
  error::SdkError,
  operation::{get_item::GetItemError, put_item::PutItemError},
  types::AttributeValue,
};
use mongodb::bson::{doc, to_document};
use thiserror::Error;

use crate::{
  shared::database::{DynamoDatabase, MongoDatabase},
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

// ### DynamoDB implementation ###

pub struct DynamoUserRepositoryImpl {
  database: DynamoDatabase,
}

impl DynamoUserRepositoryImpl {
  pub fn new(database: DynamoDatabase) -> Self {
    Self { database }
  }
}

impl UserRepository for DynamoUserRepositoryImpl {
  async fn find_one<'a>(
    &self,
    property: FindOneProperty<'a>,
  ) -> Result<User, UserRepositoryError> {
    let (key, value) = property.to_key_value();
    let result = self
      .database
      .dynamo_client
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
      .dynamo_client
      .put_item()
      .table_name("users")
      .set_item(Some(item))
      .send()
      .await?;
    Ok(())
  }
}

// ### MongoDB implementation ###

pub struct MongoUserRepositoryImpl {
  database: MongoDatabase,
}

impl MongoUserRepositoryImpl {
  pub fn new(database: MongoDatabase) -> Self {
    Self { database }
  }
}

impl UserRepository for MongoUserRepositoryImpl {
  async fn find_one<'a>(
    &self,
    property: FindOneProperty<'a>,
  ) -> Result<User, UserRepositoryError> {
    let result: Option<User> = self
      .database
      .mongo_client
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
      .mongo_client
      .database("test")
      .collection("users")
      .insert_one(to_document(&user).unwrap())
      .await;
    Ok(())
  }
}

#[cfg(test)]
pub mod tests {
  use super::{FindOneProperty, UserRepository, UserRepositoryError};
  use crate::users::model::user::User;
  use std::sync::{Arc, RwLock};

  pub struct InMemoryUserRepository {
    pub users: Arc<RwLock<Vec<User>>>,
  }

  impl InMemoryUserRepository {
    pub fn new() -> Self {
      Self {
        users: Arc::new(RwLock::new(Vec::new())),
      }
    }
  }

  impl UserRepository for InMemoryUserRepository {
    async fn find_one<'a>(
      &self,
      property: FindOneProperty<'a>,
    ) -> Result<User, UserRepositoryError> {
      let users = self.users.read().unwrap(); // Acquire read lock

      let result = users
        .iter()
        .find(|user| match property {
          FindOneProperty::Uuid(uuid) => user.uuid == uuid,
          FindOneProperty::Email(email) => user.email == email,
        })
        .cloned();
      result.ok_or(UserRepositoryError::Other(String::new()))
    }

    async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
      let mut users = self.users.write().unwrap(); // Acquire write lock
      users.push(user.clone());
      Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError> {
      Ok(self.users.read().unwrap().clone())
    }
  }
}
