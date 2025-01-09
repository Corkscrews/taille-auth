pub mod dto;

use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::DEFAULT_COST;
use bcrypt::{hash, BcryptError};
use dto::create_user_dto::CreateUserDto;
use nanoid::nanoid;
use validator::Validate;

use crate::shared::http_error::HttpError;
use crate::shared::model::user::User;
use crate::shared::repository::user_repository::{
  FindOneProperty, UserRepository,
};
use crate::shared::rto::created_rto::CreatedRto;
use crate::AppState;

pub async fn create_user<UR: UserRepository>(
  data: web::Data<AppState<UR>>,
  dto: web::Json<CreateUserDto>,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // TODO: This solution below is vulnerable to time based attacks, transform the login
  // process into a time constant solution to prevent those issues.
  // Call `find_one` with `await` on the repository instance
  let user = data
    .user_repository
    .find_one(FindOneProperty::Email(&dto.email))
    .await;

  if user.is_ok() {
    return user_already_exists();
  }

  // Start a new async block. The closure is blocking and is ochestrated by the
  // thread-pool.
  let password_hash_result = hash_password(&data, &dto).await;

  if let Err(error) = password_hash_result {
    eprintln!("{}", error);
    return internal_server_error();
  }
  let password_hash = password_hash_result.unwrap();
  // Create a domain User from the DTO.
  let user = User::from(dto.into_inner(), password_hash);

  data
    .user_repository
    .create(user.clone())
    .await
    .map(|_| {
      HttpResponse::Created()
        .content_type("application/json")
        .append_header((header::LOCATION, format!("/v1/users/{}", &user.uuid)))
        .json(CreatedRto::from(user.uuid.as_ref()))
    })
    .unwrap_or_else(|error| {
      eprintln!("{}", error);
      internal_server_error()
    })
}

async fn hash_password<UR: UserRepository>(
  data: &web::Data<AppState<UR>>,
  dto: &CreateUserDto,
) -> Result<String, BcryptError> {
  // Create a closed reference of the password to be passed to the thread-pool runner.
  let password = dto.password.clone();
  // Start a new async block. The closure is blocking and is ochestrated by the
  // thread-pool.
  web::block({
    let thread_pool = data.thread_pool.clone();
    let password = password.to_owned();
    move || {
      let thread_pool = thread_pool.lock().unwrap();
      thread_pool.install(|| hash(password, DEFAULT_COST))
    }
  })
  .await
  .unwrap()
}

fn user_already_exists() -> HttpResponse {
  HttpResponse::Conflict()
    .content_type("application/json")
    .json(HttpError::from("User already exists"))
}

fn internal_server_error() -> HttpResponse {
  HttpResponse::InternalServerError().finish()
}

impl User {
  fn from(dto: CreateUserDto, password_hash: String) -> Self {
    Self {
      uuid: nanoid!(),
      email: dto.email,
      user_name: dto.user_name,
      password_hash,
      role: dto.role,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, Mutex, RwLock};

  use actix_web::{http::StatusCode, HttpRequest};
  use fake::{
    faker::{
      internet::en::{Password, SafeEmail},
      name::raw::Name,
    },
    locales::EN,
    Fake,
  };
  use nanoid::nanoid;
  use rayon::ThreadPoolBuilder;

  use crate::{
    helpers::tests::{http_request, parse_http_response},
    shared::{
      config::Config,
      repository::user_repository::tests::InMemoryUserRepository, role::Role,
    },
  };

  use super::*;

  #[actix_web::test]
  async fn test_create_user_successful() {
    let jwt_secret = nanoid!();

    let dto = CreateUserDto {
      email: SafeEmail().fake(),
      user_name: Name(EN).fake(),
      password: Password(12..13).fake(),
      role: Role::Customer,
    };

    let users = Arc::new(RwLock::new(Vec::new()));

    let app_state = AppState {
      user_repository: InMemoryUserRepository {
        users: users.clone(),
      },
      config: Config {
        master_key: nanoid!(),
        jwt_secret: jwt_secret.clone(),
      },
      thread_pool: Arc::new(Mutex::new(
        ThreadPoolBuilder::new().num_threads(1).build().unwrap(),
      )),
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder =
      create_user(web::Data::new(app_state), web::Json(dto)).await;

    let rto: CreatedRto =
      parse_http_response(responder, &request, StatusCode::CREATED).await;

    let users = users.read().unwrap().clone();
    assert!(!users.is_empty());

    // Assertions
    assert_eq!(rto.uuid, users[0].uuid);
  }

  #[actix_web::test]
  async fn test_create_user_already_exists() {
    let jwt_secret = nanoid!();

    let dto = CreateUserDto {
      email: SafeEmail().fake(),
      user_name: Name(EN).fake(),
      password: Password(12..13).fake(),
      role: Role::Customer,
    };

    let users = Arc::new(RwLock::new(vec![
      User::from(dto.clone(), String::new())
    ]));

    let app_state = AppState {
      user_repository: InMemoryUserRepository {
        users: users.clone(),
      },
      config: Config {
        master_key: nanoid!(),
        jwt_secret: jwt_secret.clone(),
      },
      thread_pool: Arc::new(Mutex::new(
        ThreadPoolBuilder::new().num_threads(1).build().unwrap(),
      )),
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder =
      create_user(web::Data::new(app_state), web::Json(dto)).await;

    let users = users.read().unwrap().clone();
    assert_eq!(users.len(), 1);

    let error: HttpError = parse_http_response(
      responder, 
      &request, 
      StatusCode::CONFLICT
    ).await;

    // Assertions
    assert_eq!(error.message, "User already exists");
  }
}
