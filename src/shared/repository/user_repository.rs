use std::sync::Arc;
use aws_sdk_dynamodb::{error::SdkError, operation::{get_item::GetItemError, put_item::PutItemError}, types::AttributeValue};
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

pub trait UserRepository {
  async fn find_one(&self, email: &str) -> Result<User, UserRepositoryError>;
  async fn create(&self, user: User) -> Result<(), UserRepositoryError>;
}

pub struct UserRepositoryImpl {
  database: Arc<Database>
}

impl UserRepositoryImpl {
  pub fn new(database: Arc<Database>) -> Self {
    Self {
      database: database.clone()
    }
  }
}

impl UserRepository for UserRepositoryImpl {
  async fn find_one(&self, email: &str) -> Result<User, UserRepositoryError> {
    let result = self.database.dynamo_client
      .get_item()
      .table_name("users")
      .key("email", AttributeValue::S(email.to_string()))
      .send()
      .await?;

    if let Some(item) = result.item {
      let user: User = serde_dynamo::from_item(item).unwrap();
      return Ok(user);
    }
    return Err(UserRepositoryError::Other(String::from("No item")))
  }

  async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
    let item = serde_dynamo::to_item(&user).unwrap();
    self.database.dynamo_client
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
  use super::{UserRepository, UserRepositoryError};
  use crate::shared::model::user::User;
  use std::sync::RwLock;

  pub struct UserRepositoryMock {
    users: RwLock<Vec<User>>,
  }

  impl UserRepositoryMock {
    pub fn new() -> Self {
      Self {
        users: RwLock::new(Vec::new()),
      }
    }
  }

  impl UserRepository for UserRepositoryMock {
      async fn find_one(&self, email: &str) -> Result<User, UserRepositoryError> {
      let users = self.users.read().unwrap(); // Acquire read lock
      let result = users.iter().find(|user| user.email == email).cloned();
      result.ok_or(UserRepositoryError::Other(String::new()))
    }

    async fn create(&self, user: User) -> Result<(), UserRepositoryError> {
      let mut users = self.users.write().unwrap(); // Acquire write lock
      users.push(user.clone());
      Ok(())
    }
  }
}
