use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use actix_web::{web, HttpResponse, Responder};
use bcrypt::verify;
use dto::login_dto::LoginDTO;
use jsonwebtoken::encode;
use jsonwebtoken::Algorithm;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;

use crate::shared::repository::user_repository::UserRepository;
use crate::shared::role::Role;
use crate::AppState;

mod dto;
mod rto;

const ACCESS_TOKEN_EXPIRY: u64 = 15 * 60; // 15 minutes in seconds
const REFRESH_TOKEN_EXPIRY: u64 = 7 * 24 * 60 * 60; // 7 days in seconds

#[derive(Serialize, Deserialize)]
struct AccessTokenClaims {
  id: u32,
  role: Role,
  sub: String,
  iat: u64,
  exp: u64,
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenClaims {
  id: u32,
  iat: u64,
  exp: u64,
}

#[derive(Serialize)]
struct TokenResponse {
  access_token: String,
  refresh_token: String,
}

// #[post("login")]
pub async fn auth_login<UR: UserRepository>(
  data: web::Data<AppState<UR>>,
  payload: web::Json<LoginDTO>,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = payload.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // TODO: This solution below is vulnerable to time based attacks, transform the login
  // process into a time constant solution to prevent those issues.
  // Call `find_one` with `await` on the repository instance
  let user = data.user_repository.find_one(&payload.username).await;

  if user.is_none() {
    return unauthorized();
  }
  let user = user.unwrap();

  // Verify the password
  let password_is_valid =
    verify(payload.password.clone(), &user.password).unwrap_or(false);

  if !password_is_valid {
    return unauthorized();
  }

  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

  // Generate tokens
  let access_token = generate_jwt(
    &data,
    AccessTokenClaims {
      id: user.id,
      role: user.role,
      sub: user.user_name.clone(),
      iat: now,
      exp: now + ACCESS_TOKEN_EXPIRY,
    },
  );
  let refresh_token = generate_jwt(
    &data,
    RefreshTokenClaims {
      id: user.id,
      iat: now,
      exp: now + REFRESH_TOKEN_EXPIRY,
    },
  );

  if access_token.is_err() || refresh_token.is_err() {
    return HttpResponse::InternalServerError().finish();
  }

  let tokens = TokenResponse {
    access_token: access_token.unwrap(),
    refresh_token: refresh_token.unwrap(),
  };

  HttpResponse::Ok()
    .content_type("application/json")
    .json(tokens)
}

fn generate_jwt<T: Serialize, UR: UserRepository>(
  data: &web::Data<AppState<UR>>,
  claims: T,
) -> Result<String, jsonwebtoken::errors::Error> {
  encode(
    &Header::new(Algorithm::HS256),
    &claims,
    &EncodingKey::from_secret(data.config.jwt_secret.as_ref()),
  )
}

fn unauthorized() -> HttpResponse {
  HttpResponse::Unauthorized()
    .content_type("application/json")
    .body("Unauthorized")
}
