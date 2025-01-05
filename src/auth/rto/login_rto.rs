use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginRTO {
  pub message: String,
}
