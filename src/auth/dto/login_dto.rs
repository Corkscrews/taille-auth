use serde::Deserialize;
use utoipa::ToSchema;
use validator_derive::Validate;

#[derive(ToSchema, Debug, Deserialize, Validate)]
pub struct LoginDto {
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
