pub mod dto;

use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::hash;
use bcrypt::DEFAULT_COST;
use dto::create_user_dto::CreateUserDTO;
use validator::Validate;

use crate::shared::model::user::User;
use crate::shared::repository::user_repository::UserRepository;
use crate::AppState;

// #[post("login")]
pub async fn create_user<UR: UserRepository>(
  data: web::Data<AppState<UR>>,
  payload: web::Json<CreateUserDTO>,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = payload.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // TODO: This solution below is vulnerable to time based attacks, transform the login
  // process into a time constant solution to prevent those issues.
  // Call `find_one` with `await` on the repository instance
  let user = data.user_repository.find_one(&payload.user_name).await;

  if user.is_some() {
    return user_already_exists();
  }

  let user = data.user_repository.create(User::from(payload.0)).await;

  HttpResponse::Created()
    .content_type("application/json")
    .append_header((header::LOCATION, format!("/v1/users/{}", user.id)))
    .body(r#"{"message": "Resource created"}"#)
}

fn user_already_exists() -> HttpResponse {
  HttpResponse::Unauthorized()
    .content_type("application/json")
    .body(r#"{"message": "User already exists"}"#)
}

impl From<CreateUserDTO> for User {
  fn from(dto: CreateUserDTO) -> Self {
    let password_hash = hash(dto.password, DEFAULT_COST).unwrap();
    Self {
      id: 123,
      user_name: dto.user_name,
      password: password_hash,
      role: dto.role,
    }
  }
}
