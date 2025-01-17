use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Role {
  #[serde(rename = "admin")]
  Admin,
  #[serde(rename = "manager")]
  Manager,
  #[serde(rename = "driver")]
  Driver,
  #[serde(rename = "customer")]
  Customer,
}
