use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDTO {
  #[validate(length(
    min = 1,
    message = "email must have at least 1 characters"
  ))]
  pub email: String,
  #[validate(length(
    min = 1,
    message = "password must have at least 1 characters"
  ))]
  pub password: String,
}
