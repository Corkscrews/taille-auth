mod auth;
mod helpers;
mod shared;
mod users;

use std::{
  cmp::max,
  sync::{Arc, LazyLock},
};

use actix_governor::{
  governor::{clock::QuantaInstant, middleware::NoOpMiddleware},
  Governor, GovernorConfig, GovernorConfigBuilder, PeerIpKeyExtractor,
};
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use nanoid::nanoid;
use rayon::ThreadPoolBuilder;
use shared::{
  config::Config,
  database::resolve_database,
  handlers::check_health,
  hash_worker::{HashWorker, Hasher},
  health_check::{HealthCheck, HealthCheckImpl},
  middleware::master_key_middleware::bearer_validator,
};
use utoipa::OpenApi;

use auth::handlers::{access_token, auth_login};
use users::{
  handlers::{create_user, get_users},
  repository::user_repository::{UserRepository, UserRepositoryImpl},
};
use utoipa_scalar::{Scalar, Servable};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  println!("Starting taille-auth...");
  let config = Config::default().await;

  let database = Arc::new(resolve_database(&config).await);
  let health_check = Arc::new(HealthCheckImpl::new(database.clone()));

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

  let address = config.address.clone();

  let config = Arc::new(config);

  let http_server = HttpServer::new(move || {
    App::new().configure(|cfg| {
      apply_service_config(
        cfg,
        &governor_config,
        config.clone(),
        health_check.clone(),
        hasher.clone(),
        UserRepositoryImpl::new(database.clone())
      )
    })
  })
  .workers(2)
  .bind(address.clone())?
  .run();

  println!("Listening on http://{}", address);
  http_server.await
}

// Function to initialize the App
fn apply_service_config<
  UR: UserRepository + 'static,
  HC: HealthCheck + 'static,
  H: Hasher + 'static,
>(
  service_config: &mut web::ServiceConfig,
  governor_config: &GovernorConfig<
    PeerIpKeyExtractor,
    NoOpMiddleware<QuantaInstant>,
  >,
  config: Arc<Config>,
  health_check: Arc<HC>,
  hasher: Arc<H>,
  user_repository: UR,
) {
  service_config
    .app_data(web::Data::from(config.clone()))
    .app_data(web::Data::from(health_check.clone()))
    .app_data(web::Data::new(user_repository))
    .app_data(web::Data::from(hasher))
    .service(Scalar::with_url("/docs", ApiDoc::openapi()))
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
        )
        .service(
          web::scope("/health").route("", web::get().to(check_health::<HC>)),
        ),
    );
}

fn num_threads() -> usize {
  std::thread::available_parallelism().unwrap().get()
}

static CUSTOM_ALPHABET: LazyLock<Vec<char>> = LazyLock::new(|| {
  nanoid::alphabet::SAFE
    .iter()
    .filter(|&&c| c != '_' && c != '-')
    .copied()
    .collect()
});

fn custom_nanoid() -> String {
  // Generate a nanoid with the custom alphabet and desired size
  nanoid!(21, &*CUSTOM_ALPHABET)
}

#[derive(OpenApi)]
#[openapi(paths(
  crate::auth::handlers::auth_login,
  crate::auth::handlers::access_token,
  crate::users::handlers::get_users,
  crate::users::handlers::create_user,
  crate::shared::handlers::check_health
))]
struct ApiDoc;

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
  use shared::database::{Database, InMemoryDatabase};
  use std::{env, net::SocketAddr, str::FromStr, time::Duration};
  use users::repository::user_repository::UserRepositoryImpl;

  #[actix_rt::test]
  async fn test_create_user_and_login_in_memory() {
    let master_key = String::from("FAKE_MASTER_KEY");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    let config = Arc::new(Config::default().await);
    let database = Arc::new(InMemoryDatabase::new(&config).await.unwrap());
    let health_check = Arc::new(HealthCheckImpl::new(database.clone()));

    // Initialize the service in-memory
    let app = test::init_service(App::new().configure(|cfg| {
      apply_service_config(
        cfg,
        &GovernorConfigBuilder::default().finish().unwrap(),
        config,
        health_check,
        Arc::new(HashWorker::new(
          ThreadPoolBuilder::new()
            .num_threads(max(num_threads() - 2, 1))
            .build()
            .unwrap(),
          2,
        )),
        UserRepositoryImpl::new(database.clone()),
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
