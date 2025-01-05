mod auth;
mod shared;
mod user;

use actix_web::{web, App, HttpServer};
use auth::auth_login;
use shared::user_repository::{UserRepository, UserRepositoryImpl};
use user::create_user;

// This struct represents state
struct AppState<UR: UserRepository> {
  user_repository: UR,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_address = "127.0.0.1:3000";
  println!("Listening on http://{}", server_address);

  HttpServer::new(|| {
    App::new()
      .app_data(web::Data::new(AppState {
        user_repository: UserRepositoryImpl::new(),
      }))
      .service(
        web::scope("/v1")
          .service(
            web::scope("/auth")
              .route("login", web::post().to(auth_login::<UserRepositoryImpl>)),
          )
          .service(
            web::scope("/user")
              .route("", web::post().to(create_user::<UserRepositoryImpl>)),
          ),
      )
    // .route("/v1/auth/login", web::post().to(login_handler))
  })
  .bind(server_address)?
  .run()
  .await
}
