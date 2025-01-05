use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDTO {
  #[validate(length(
    min = 1,
    message = "UserName must have at least 1 characters"
  ))]
  #[serde(rename = "userName")]
  pub user_name: String,
  #[validate(length(
    min = 1,
    message = "Password must have at least 1 characters"
  ))]
  pub password: String,
}
