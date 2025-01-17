use actix_web::{HttpResponse, Responder};

pub mod config;
pub mod database;
pub mod hash_worker;
pub mod http_error;
pub mod middleware;
pub mod role;
pub mod rto;

#[utoipa::path(
  post,
  path = "/health",
  responses(
      (status = 200, description = "Check the service health")
  )
)]
pub async fn check_health() -> impl Responder {
  HttpResponse::Ok().finish()
}
