mod auth;
mod config;
mod shared;
mod user;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use auth::auth_login;
use config::Config;
use shared::{
  middleware::master_key_middleware::bearer_validator,
  repository::user_repository::{UserRepository, UserRepositoryImpl},
};
use user::create_user;

// This struct represents state
struct AppState<UR: UserRepository> {
  user_repository: UR,
  config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_address = "127.0.0.1:3000";
  println!("Listening on http://{}", server_address);
  HttpServer::new(|| App::new().configure(config))
    .bind(server_address)?
    .run()
    .await
}

// Function to initialize the App
fn config(config: &mut web::ServiceConfig) {
  // Rate limit
  // Allow bursts with up to five requests per IP address
  // and replenishes two elements per second
  let governor_config = GovernorConfigBuilder::default()
    .requests_per_second(2)
    .burst_size(5)
    .finish()
    .unwrap();

  config
    .app_data(web::Data::new(AppState {
      user_repository: UserRepositoryImpl::new(),
      config: Config::default(),
    }))
    .service(
      web::scope("/v1")
        .service(
          web::scope("/auth")
            .wrap(Governor::new(&governor_config))
            .route("login", web::post().to(auth_login::<UserRepositoryImpl>)),
        )
        .service(
          web::scope("/user")
            .wrap(HttpAuthentication::with_fn(
              bearer_validator::<UserRepositoryImpl>,
            ))
            .route("", web::post().to(create_user::<UserRepositoryImpl>)),
        ),
    );
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::{http::header::HeaderValue, test, App};
  use std::{env, net::SocketAddr, str::FromStr};

  #[actix_rt::test]
  async fn test_create_user_and_login_in_memory() {
    let master_key = String::from("FAKE_MASTER_KEY");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    // Initialize the service in-memory
    let app = test::init_service(
      App::new().configure(config), // your config function
    )
    .await;

    // 1) Create user
    let create_req = test::TestRequest::post()
      .uri("/v1/user")
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", master_key)).unwrap(),
      ))
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .set_json(serde_json::json!({
          "userName": "testuser",
          "password": "testpassword",
          "role": "driver"
      }))
      .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    println!("{:?}", create_resp.response().body());
    assert!(create_resp.status().is_success(), "Create user failed");

    // 2) Login
    let login_req = test::TestRequest::post()
      .uri("/v1/auth/login")
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .set_json(serde_json::json!({
          "userName": "testuser",
          "password": "testpassword"
      }))
      .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success(), "Login failed");

    let body_bytes = test::read_body(login_resp).await;
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    println!("Login response: {}", body_str);
  }
}
