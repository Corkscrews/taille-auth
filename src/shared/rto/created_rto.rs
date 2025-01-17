use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct CreatedRto {
  pub uuid: String,
}
