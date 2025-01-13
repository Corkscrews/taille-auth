use actix_web::HttpRequest;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use dto::login_dto::LoginDto;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use rto::login_rto::LoginRto;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;

use crate::shared::config::Config;
use crate::shared::hash_worker::Hasher;
use crate::shared::http_error::HttpError;
use crate::shared::role::Role;
use crate::users::model::user::User;
use crate::users::repository::user_repository::FindOneProperty;
use crate::users::repository::user_repository::UserRepository;

pub mod dto;
pub mod rto;

const ACCESS_TOKEN_EXPIRY: u64 = 15 * 60; // 15 minutes in seconds
const REFRESH_TOKEN_EXPIRY: u64 = 7 * 24 * 60 * 60; // 7 days in seconds

#[derive(Serialize, Deserialize)]
struct AccessTokenClaims {
  uuid: String,
  role: Role,
  sub: String,
  iat: u64,
  exp: u64,
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenClaims {
  uuid: String,
  iat: u64,
  exp: u64,
}

pub async fn auth_login<UR: UserRepository, H: Hasher>(
  config: web::Data<Config>,
  user_repository: web::Data<UR>,
  hasher: web::Data<H>,
  dto: web::Json<LoginDto>,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // TODO: This solution below is vulnerable to time based attacks, transform the login
  // process into a time constant solution to prevent those issues.
  // Call `find_one` with `await` on the repository instance
  let user = user_repository
    .find_one(FindOneProperty::Email(&dto.email))
    .await;
  if user.is_err() {
    return unauthorized();
  }
  let user = user.unwrap();

  let password_match_result = hasher
    .as_ref()
    .verify_password(&dto.password, &user.password_hash)
    .await;

  if !password_match_result.unwrap_or(false) {
    return unauthorized();
  }
  generate_token_response(&config, user)
}

pub async fn access_token<UR: UserRepository + 'static, H: Hasher>(
  config: web::Data<Config>,
  user_repository: web::Data<UR>,
  request: HttpRequest,
) -> impl Responder {
  let refresh_token_claims = decode_refresh_token(&config, request).await;
  if refresh_token_claims.is_none() {
    return unauthorized();
  }
  let refresh_token_claims = refresh_token_claims.unwrap();

  let user = user_repository
    .find_one(FindOneProperty::Uuid(&refresh_token_claims.uuid))
    .await;
  if user.is_err() {
    return unauthorized();
  }
  let user = user.unwrap();

  generate_token_response(&config, user)
}

async fn decode_refresh_token(
  config: &Config,
  request: HttpRequest,
) -> Option<RefreshTokenClaims> {
  // Extract the Authorization header
  let authorization_header = match request.headers().get("Authorization") {
    Some(header_value) => match header_value.to_str() {
      Ok(value) => value,
      Err(_) => return None,
    },
    None => return None,
  };
  let token = authorization_header.replace("Bearer ", "");

  let decode_result = decode::<RefreshTokenClaims>(
    &token,
    &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
    &Validation::default(),
  );

  if decode_result.is_err() {
    return None;
  }
  let decode_result = decode_result.unwrap();

  Some(decode_result.claims)
}

fn generate_jwt<T: Serialize>(
  config: &Config,
  claims: T,
) -> Result<String, jsonwebtoken::errors::Error> {
  encode(
    &Header::new(Algorithm::HS256),
    &claims,
    &EncodingKey::from_secret(config.jwt_secret.as_ref()),
  )
}

fn generate_token_response(config: &Config, user: User) -> HttpResponse {
  let now = Utc::now().timestamp() as u64;

  // Generate tokens
  let access_token = generate_jwt(
    config,
    AccessTokenClaims {
      uuid: user.uuid.clone(),
      role: user.role,
      sub: user.user_name.clone(),
      iat: now,
      exp: now + ACCESS_TOKEN_EXPIRY,
    },
  );
  let refresh_token = generate_jwt(
    config,
    RefreshTokenClaims {
      uuid: user.uuid.clone(),
      iat: now,
      exp: now + REFRESH_TOKEN_EXPIRY,
    },
  );

  if access_token.is_err() || refresh_token.is_err() {
    return HttpResponse::InternalServerError().finish();
  }

  let tokens = LoginRto {
    access_token: access_token.unwrap(),
    refresh_token: refresh_token.unwrap(),
  };

  HttpResponse::Ok()
    .content_type("application/json")
    .json(tokens)
}

fn unauthorized() -> HttpResponse {
  HttpResponse::Unauthorized()
    .content_type("application/json")
    .json(HttpError::from("Unauthorized"))
}
