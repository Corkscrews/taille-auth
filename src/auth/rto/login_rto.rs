use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginRto {
  #[serde(rename = "accessToken")]
  pub access_token: String,
  #[serde(rename = "refreshToken")]
  pub refresh_token: String,
}
