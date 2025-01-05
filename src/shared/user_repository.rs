use std::sync::RwLock;

use super::model::user::User;

pub trait UserRepository {
  async fn find_one(&self, username: &str) -> Option<User>;
  async fn create(&self, user: User) -> User;
}

pub struct UserRepositoryImpl {
  users: RwLock<Vec<User>>,
}

impl UserRepositoryImpl {
  pub fn new() -> Self {
    Self {
      users: RwLock::new(Vec::new()),
    }
  }
}

impl UserRepository for UserRepositoryImpl {
  async fn find_one(&self, username: &str) -> Option<User> {
    let users = self.users.read().unwrap(); // Acquire read lock
    users
      .iter()
      .find(|user| user.user_name == username)
      .cloned()
  }
  async fn create(&self, user: User) -> User {
    let mut users = self.users.write().unwrap(); // Acquire write lock
    users.push(user.clone());
    user
  }
}
