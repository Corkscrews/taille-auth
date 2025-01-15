use actix_web::{HttpResponse, Responder};

pub mod config;
pub mod database;
pub mod hash_worker;
pub mod http_error;
pub mod middleware;
pub mod role;
pub mod rto;

pub async fn check_health() -> impl Responder {
  HttpResponse::Ok().finish()
}