use serde::{Deserialize, Serialize};

use crate::shared::role::Role;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  pub uuid: String,
  pub email: String,
  pub user_name: String,
  pub password_hash: String,
  pub role: Role,
}
