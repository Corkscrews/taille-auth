#[cfg(test)]
pub mod tests {
  use crate::shared::role::Role;
  use actix_web::{
    http::{header::HeaderValue, StatusCode},
    HttpRequest, Responder,
  };
  use fake::{faker::internet::en::SafeEmail, Fake};
  use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
  use nanoid::nanoid;
  use serde::{de::DeserializeOwned, Deserialize, Serialize};

  #[derive(Serialize, Deserialize)]
  pub struct FakeAccessTokenClaims {
    uuid: String,
    role: Role,
    sub: String,
    iat: u64,
    exp: u64,
  }

  pub fn create_fake_access_token(jwt_secret: &str) -> String {
    let fake_claims = FakeAccessTokenClaims {
      uuid: nanoid!(),
      role: Role::Manager,
      sub: SafeEmail().fake(),
      iat: 0,
      exp: 253402300799,
    };
    encode(
      &Header::new(Algorithm::HS256),
      &fake_claims,
      &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap()
  }

  pub fn http_request(jwt_secret: &str) -> HttpRequest {
    let authorization_header = HeaderValue::from_str(&format!(
      "Bearer {}",
      create_fake_access_token(&jwt_secret)
    ))
    .unwrap();
    actix_web::test::TestRequest::default()
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header.clone(),
      ))
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .to_http_request()
  }

  pub async fn parse_http_response<T: DeserializeOwned>(
    responder: impl Responder,
    request: &HttpRequest,
    status_code: StatusCode,
  ) -> T {
    // Convert the `Responder` into an HttpResponse
    let http_response = responder.respond_to(&request);

    // Wrap the HttpResponse in a ServiceResponse so that test utilities can work with it
    let service_response =
      actix_web::test::TestRequest::default().to_srv_response(http_response);

    // Now you can use `read_body_json` on the `ServiceResponse`:
    assert_eq!(service_response.status(), status_code);
    println!("{:?}", service_response.response());
    actix_web::test::read_body_json(service_response).await
  }
}
