use crate::shared::role::Role;

#[derive(Clone)]
pub struct User {
  pub id: u32,
  pub user_name: String,
  pub password: String,
  pub role: Role,
}
