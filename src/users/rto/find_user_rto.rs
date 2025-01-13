use serde::{Deserialize, Serialize};

use crate::shared::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindUserRto {
  pub email: String,
  pub user_name: String,
  pub role: Role,
}
