use serde::{Deserialize, Serialize};

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
