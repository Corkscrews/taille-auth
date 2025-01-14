pub mod dto;
pub mod model;
pub mod repository;
pub mod rto;

use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use dto::create_user_dto::CreateUserDto;
use rto::find_user_rto::FindUserRto;
use validator::Validate;

use crate::custom_nanoid;
use crate::shared::hash_worker::Hasher;
use crate::shared::http_error::HttpError;
use crate::shared::rto::created_rto::CreatedRto;
use crate::users::model::user::User;
use crate::users::repository::user_repository::{
  FindOneProperty, UserRepository,
};

pub async fn create_user<UR: UserRepository, H: Hasher>(
  user_repository: web::Data<UR>,
  hasher: web::Data<H>,
  dto: web::Json<CreateUserDto>,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  let user = user_repository
    .find_one(FindOneProperty::Email(&dto.email))
    .await;

  if user.is_ok() {
    return user_already_exists();
  }

  let password_hash_result = hasher.as_ref().hash_password(&dto.password).await;

  if let Err(error) = password_hash_result {
    eprintln!("{}", error);
    return internal_server_error();
  }
  let password_hash = password_hash_result.unwrap();
  // Create a domain User from the DTO.
  let user = User::from(dto.into_inner(), password_hash);

  user_repository
    .create(user.clone())
    .await
    .map(|_| {
      HttpResponse::Created()
        .content_type("application/json")
        .append_header((header::LOCATION, format!("/v1/users/{}", &user.uuid)))
        .json(CreatedRto::from(user))
    })
    .unwrap_or_else(|error| {
      eprintln!("{}", error);
      internal_server_error()
    })
}

pub async fn get_users<UR: UserRepository>(
  user_repository: web::Data<UR>,
) -> impl Responder {
  user_repository
    .find_all()
    .await
    .map(|users| {
      HttpResponse::Created()
        .content_type("application/json")
        .json(
          users
            .into_iter()
            .map(FindUserRto::from)
            .collect::<Vec<FindUserRto>>(),
        )
    })
    .unwrap_or_else(|error| {
      eprintln!("{}", error);
      internal_server_error()
    })
}

impl From<User> for FindUserRto {
  fn from(user: User) -> Self {
    Self {
      email: user.email,
      user_name: user.user_name,
      role: user.role,
    }
  }
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
      uuid: custom_nanoid(),
      email: dto.email,
      user_name: dto.user_name,
      password_hash,
      role: dto.role,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl From<User> for CreatedRto {
  fn from(user: User) -> Self {
    Self { uuid: user.uuid }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, RwLock};

  use actix_web::{http::StatusCode, HttpRequest};
  use fake::{
    faker::{
      internet::en::{Password, SafeEmail},
      name::raw::Name,
    },
    locales::EN,
    Fake,
  };
  use rayon::ThreadPoolBuilder;
  use repository::user_repository::UserRepositoryImpl;

  use crate::{
    custom_nanoid,
    helpers::tests::{http_request, parse_http_response},
    shared::{database::InMemoryDatabase, hash_worker::HashWorker, role::Role},
  };

  use super::*;

  #[actix_web::test]
  async fn test_create_user_successful() {
    let jwt_secret = custom_nanoid();

    let dto = CreateUserDto {
      email: SafeEmail().fake(),
      user_name: Name(EN).fake(),
      password: Password(12..13).fake(),
      role: Role::Customer,
    };

    let users = Arc::new(RwLock::new(Vec::new()));

    let user_repository =
      UserRepositoryImpl::<InMemoryDatabase>::new(InMemoryDatabase {
        users: users.clone(),
      });

    let hasher = HashWorker::new(ThreadPoolBuilder::new().build().unwrap(), 2);

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = create_user(
      web::Data::new(user_repository),
      web::Data::new(hasher),
      web::Json(dto),
    )
    .await;

    let rto: CreatedRto =
      parse_http_response(responder, &request, StatusCode::CREATED).await;

    let users = users.read().unwrap().clone();
    assert!(!users.is_empty());

    // Assertions
    assert_eq!(rto.uuid, users[0].uuid);
  }

  #[actix_web::test]
  async fn test_create_user_already_exists() {
    let jwt_secret = custom_nanoid();

    let dto = CreateUserDto {
      email: SafeEmail().fake(),
      user_name: Name(EN).fake(),
      password: Password(12..13).fake(),
      role: Role::Customer,
    };

    let users =
      Arc::new(RwLock::new(vec![User::from(dto.clone(), String::new())]));

    let user_repository =
      UserRepositoryImpl::<InMemoryDatabase>::new(InMemoryDatabase {
        users: users.clone(),
      });

    let hasher = HashWorker::new(ThreadPoolBuilder::new().build().unwrap(), 2);

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = create_user(
      web::Data::new(user_repository),
      web::Data::new(hasher),
      web::Json(dto),
    )
    .await;

    let users = users.read().unwrap().clone();
    assert_eq!(users.len(), 1);

    let error: HttpError =
      parse_http_response(responder, &request, StatusCode::CONFLICT).await;

    // Assertions
    assert_eq!(error.message, "User already exists");
  }

  #[actix_web::test]
  async fn test_create_user_validation_failure() {
    let jwt_secret = custom_nanoid();

    let dto = CreateUserDto {
      email: "invalid_email".to_string(),
      user_name: "".to_string(),
      password: "short".to_string(),
      role: Role::Customer,
    };

    let users = Arc::new(RwLock::new(Vec::new()));

    let user_repository =
      UserRepositoryImpl::<InMemoryDatabase>::new(InMemoryDatabase {
        users: users.clone(),
      });

    let hasher = HashWorker::new(ThreadPoolBuilder::new().build().unwrap(), 2);

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = create_user(
      web::Data::new(user_repository),
      web::Data::new(hasher),
      web::Json(dto),
    )
    .await;

    let users = users.read().unwrap().clone();
    assert!(users.is_empty());

    let error: serde_json::Value =
      parse_http_response(responder, &request, StatusCode::BAD_REQUEST).await;

    // Assertions
    println!("{}", error);
    // assert!(error.get("email").is_some());
    // assert!(error.get("user_name").is_some());
    // assert!(error.get("password").is_some());
  }

  #[actix_web::test]
  async fn test_get_users() {
    let jwt_secret = custom_nanoid();

    let users_data = vec![
      User::from(
        CreateUserDto {
          email: SafeEmail().fake(),
          user_name: Name(EN).fake(),
          password: Password(12..13).fake(),
          role: Role::Admin,
        },
        "hashed_password".to_string(),
      ),
      User::from(
        CreateUserDto {
          email: SafeEmail().fake(),
          user_name: Name(EN).fake(),
          password: Password(12..13).fake(),
          role: Role::Customer,
        },
        "hashed_password".to_string(),
      ),
    ];

    let users = Arc::new(RwLock::new(users_data.clone()));

    let user_repository =
      UserRepositoryImpl::<InMemoryDatabase>::new(InMemoryDatabase {
        users: users.clone(),
      });

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_users(web::Data::new(user_repository)).await;

    let rtos: Vec<FindUserRto> =
      parse_http_response(responder, &request, StatusCode::CREATED).await;

    // Assertions
    assert_eq!(rtos.len(), users_data.len());
    for (rto, user) in rtos.iter().zip(users_data.iter()) {
      assert_eq!(rto.email, user.email);
      assert_eq!(rto.user_name, user.user_name);
      assert_eq!(rto.role, user.role);
    }
  }

  #[actix_web::test]
  async fn test_get_users_empty() {
    let jwt_secret = custom_nanoid();

    let users = Arc::new(RwLock::new(Vec::new()));

    let user_repository =
      UserRepositoryImpl::<InMemoryDatabase>::new(InMemoryDatabase {
        users: users.clone(),
      });

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_users(web::Data::new(user_repository)).await;

    let rtos: Vec<FindUserRto> =
      parse_http_response(responder, &request, StatusCode::CREATED).await;

    // Assertions
    assert!(rtos.is_empty());
  }
}
