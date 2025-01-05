use actix_web::{dev::ServiceRequest, error, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use subtle::ConstantTimeEq;

use crate::{shared::repository::user_repository::UserRepository, AppState};

/// Validator that:
/// - accepts Bearer auth;
/// - returns a custom response for requests without a valid Bearer Authorization header;
pub async fn bearer_validator<UR: UserRepository + 'static>(
  req: ServiceRequest,
  credentials: Option<BearerAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
  let Some(credentials) = credentials else {
    return Err((error::ErrorBadRequest("no bearer header"), req));
  };

  let app_data = req.app_data::<web::Data<AppState<UR>>>().unwrap();

  // eprintln!("{credentials:?}");

  if !constant_time_compare(credentials.token(), &app_data.config.master_key) {
    return Err((error::ErrorBadRequest("Missing bearer token"), req));
  }

  Ok(req)
}

fn constant_time_compare(a: &str, b: &str) -> bool {
  a.as_bytes().ct_eq(b.as_bytes()).unwrap_u8() == 1
}
