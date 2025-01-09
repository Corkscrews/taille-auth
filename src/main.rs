mod auth;
mod helpers;
mod shared;
mod users;

use std::sync::{Arc, Mutex};

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use auth::{access_token, auth_login};
use rayon::{ThreadPool, ThreadPoolBuilder};
use shared::{
  config::Config,
  database::Database,
  middleware::master_key_middleware::bearer_validator,
  repository::user_repository::{UserRepository, UserRepositoryImpl},
};
use users::create_user;

// This struct represents state
struct AppState<UR: UserRepository> {
  user_repository: UR,
  config: Config,
  thread_pool: Arc<Mutex<ThreadPool>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_address = "127.0.0.1:3000";
  println!("Listening on http://{}", server_address);
  let database = Database::new().await;
  let database = Arc::new(database);
  HttpServer::new(move || {
    App::new()
      .configure(|cfg| config(cfg, UserRepositoryImpl::new(database.clone())))
  })
  .bind(server_address)?
  .run()
  .await
}

// Function to initialize the App
fn config<UR: UserRepository + 'static>(
  config: &mut web::ServiceConfig,
  user_repository: UR,
) {
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
      user_repository,
      config: Config::default(),
      thread_pool: Arc::new(Mutex::new(
        ThreadPoolBuilder::new().build().unwrap(),
      )),
    }))
    .service(
      web::scope("/v1")
        .service(
          web::scope("/auth")
            .wrap(Governor::new(&governor_config))
            .route("login", web::post().to(auth_login::<UR>))
            .route("access-token", web::post().to(access_token::<UR>)),
        )
        .service(
          web::scope("/users")
            .wrap(HttpAuthentication::with_fn(bearer_validator::<UR>))
            .route("", web::post().to(create_user::<UR>)),
        ),
    );
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_rt::time::sleep;
  use actix_web::{http::header::HeaderValue, test, App};
  use auth::rto::login_rto::LoginRto;
  use fake::{faker::{internet::en::{Password, SafeEmail}, name::raw::Name}, locales::EN, Fake};
  use shared::repository::user_repository::tests::InMemoryUserRepository;
  use std::{env, net::SocketAddr, str::FromStr, time::Duration};

  #[actix_rt::test]
  async fn test_create_user_and_login_in_memory() {
    let master_key = String::from("FAKE_MASTER_KEY");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    // Initialize the service in-memory
    let app = test::init_service(
      App::new().configure(|cfg| config(cfg, InMemoryUserRepository::new())),
    )
    .await;

    let email: String = SafeEmail().fake();
    let password: String = Password(12..13).fake();

    // 1) Create user
    let create_req = test::TestRequest::post()
      .uri("/v1/users")
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
        "email": email,
        "userName": Name(EN).fake::<String>(),
        "password": password,
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
        "email": email,
        "password": password
      }))
      .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success(), "Login failed");

    let login_body_bytes = test::read_body(login_resp).await;
    let login_body_str = String::from_utf8(login_body_bytes.to_vec()).unwrap();
    let login_rto: LoginRto = serde_json::from_str(&login_body_str)
      .expect("Failed to parse response JSON");

    // Required otherwise the test runs too fast and the JWT has the same second
    // expiration. Making the same JWT.
    sleep(Duration::from_secs(1)).await;

    // 3) Refresh token
    let access_token_req = test::TestRequest::post()
      .uri("/v1/auth/access-token")
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", login_rto.refresh_token))
          .unwrap(),
      ))
      .to_request();

    let access_token_resp = test::call_service(&app, access_token_req).await;
    assert!(
      access_token_resp.status().is_success(),
      "Refresh token failed"
    );

    let access_token_body_bytes = test::read_body(access_token_resp).await;
    let access_token_body_str =
      String::from_utf8(access_token_body_bytes.to_vec()).unwrap();
    let access_token_rto: LoginRto =
      serde_json::from_str(&access_token_body_str)
        .expect("Failed to parse response JSON");

    println!("access_token_rto {:?}", access_token_rto);
    println!("login_rto {:?}", login_rto);
    assert!(access_token_rto != login_rto);
  }
}
