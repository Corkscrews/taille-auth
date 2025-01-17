use actix_web::{web, HttpResponse, Responder};

use super::health_check::{HealthCheck, HealthCheckStats};

#[utoipa::path(
  post,
  path = "/health",
  responses(
    (status = 200, description = "Check the service health", body = Option<HealthCheckStats>)
  )
)]
pub async fn check_health<HC: HealthCheck>(
  check_health: web::Data<HC>,
) -> impl Responder {
  HttpResponse::Ok().json(check_health.collect())
}
