use aws_sdk_dynamodb::{
  error::SdkError,
  operation::{get_item::GetItemError, put_item::PutItemError},
  types::AttributeValue,
};
use thiserror::Error;

use crate::shared::{database::Database, model::user::User};

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
}

pub trait UserRepository {
  async fn find_one(
    &self,
    property: FindOneProperty,
  ) -> Result<User, UserRepositoryError>;
  async fn create(&self, user: User) -> Result<(), UserRepositoryError>;
}

pub struct UserRepositoryImpl {
  database: Database
}

impl UserRepositoryImpl {
  pub fn new(database: Database) -> Self {
    Self { database }
  }
}

impl UserRepository for UserRepositoryImpl {
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

#[cfg(test)]
pub mod tests {
  use super::{FindOneProperty, UserRepository, UserRepositoryError};
  use crate::shared::model::user::User;
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
  }
}
