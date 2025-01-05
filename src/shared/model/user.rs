use crate::shared::role::Role;

#[derive(Clone)]
pub struct User {
  pub uuid: String,
  pub user_name: String,
  pub password: String,
  pub role: Role,
}
