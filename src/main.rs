mod auth;
mod helpers;
mod shared;
mod users;

use std::{cmp::max, sync::Arc};

use actix_governor::{
  governor::{clock::QuantaInstant, middleware::NoOpMiddleware},
  Governor, GovernorConfig, GovernorConfigBuilder, PeerIpKeyExtractor,
};
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use auth::{access_token, auth_login};
use nanoid::nanoid;
use rayon::ThreadPoolBuilder;
use shared::{
  config::Config,
  database::{Database, InMemoryDatabase},
  hash_worker::{HashWorker, Hasher},
  middleware::master_key_middleware::bearer_validator,
};
use users::{
  create_user, get_users,
  repository::user_repository::{UserRepository, UserRepositoryImpl},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let config = Config::default().await;
  let user_repository =
    UserRepositoryImpl::new(InMemoryDatabase::new(&config).await.unwrap());
  let user_repository = Arc::new(user_repository);

  let thread_pool = ThreadPoolBuilder::new()
    .num_threads(max(num_threads() - 2, 1))
    .build()
    .unwrap();
  let hasher = Arc::new(HashWorker::new(thread_pool, 2));

  // Rate limit
  // Allow bursts with up to five requests per IP address
  // and replenishes two elements per second
  let governor_config = GovernorConfigBuilder::default()
    .requests_per_second(2)
    .burst_size(5)
    .finish()
    .unwrap();

  let server_address = config.server_address.clone();
  println!("Listening on http://{}", server_address);

  let config = Arc::new(config);

  HttpServer::new(move || {
    App::new().configure(|cfg| {
      apply_service_config(
        cfg,
        config.clone(),
        &governor_config,
        hasher.clone(),
        user_repository.clone(),
      )
    })
  })
  .workers(2)
  .bind(server_address)?
  .run()
  .await
}

// Function to initialize the App
fn apply_service_config<UR: UserRepository + 'static, H: Hasher + 'static>(
  service_config: &mut web::ServiceConfig,
  config: Arc<Config>,
  governor_config: &GovernorConfig<
    PeerIpKeyExtractor,
    NoOpMiddleware<QuantaInstant>,
  >,
  hasher: Arc<H>,
  user_repository: Arc<UR>,
) {
  service_config
    .app_data(web::Data::from(config.clone()))
    .app_data(web::Data::from(user_repository))
    .app_data(web::Data::from(hasher))
    .service(
      web::scope("/v1")
        .service(
          web::scope("/auth")
            .wrap(Governor::new(governor_config))
            .route("/login", web::post().to(auth_login::<UR, H>))
            .route("/access-token", web::post().to(access_token::<UR, H>)),
        )
        .service(
          web::scope("/users")
            .wrap(HttpAuthentication::with_fn({
              move |req, credentials| {
                bearer_validator(req, credentials, config.clone())
              }
            }))
            .route("", web::get().to(get_users::<UR>))
            .route("", web::post().to(create_user::<UR, H>)),
        ),
    );
}

fn num_threads() -> usize {
  std::thread::available_parallelism().unwrap().get()
}

lazy_static::lazy_static! {
  static ref CUSTOM_ALPHABET: Vec<char> =
    nanoid::alphabet::SAFE.iter()
      .filter(|&&c| c != '_' && c != '-')
      .copied()
      .collect();
}

fn custom_nanoid() -> String {
  // Generate a nanoid with the custom alphabet and desired size
  nanoid!(21, &*CUSTOM_ALPHABET)
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_rt::time::sleep;
  use actix_web::{http::header::HeaderValue, test, App};
  use auth::rto::login_rto::LoginRto;
  use fake::{
    faker::{
      internet::en::{Password, SafeEmail},
      name::raw::Name,
    },
    locales::EN,
    Fake,
  };
  use shared::database::InMemoryDatabase;
  use std::{env, net::SocketAddr, str::FromStr, time::Duration};
  use users::repository::user_repository::UserRepositoryImpl;

  #[actix_rt::test]
  async fn test_create_user_and_login_in_memory() {
    let master_key = String::from("FAKE_MASTER_KEY");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    let config = Config::default().await;
    let config = Arc::new(config);

    let user_repository =
      Arc::new(UserRepositoryImpl::<InMemoryDatabase>::new(
        InMemoryDatabase::new(&config).await.unwrap(),
      ));

    // Initialize the service in-memory
    let app = test::init_service(App::new().configure(|cfg| {
      apply_service_config(
        cfg,
        config,
        &GovernorConfigBuilder::default().finish().unwrap(),
        Arc::new(HashWorker::new(
          ThreadPoolBuilder::new()
            .num_threads(max(num_threads() - 2, 1))
            .build()
            .unwrap(),
          2,
        )),
        user_repository,
      )
    }))
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

    assert!(access_token_rto != login_rto);
  }
}
