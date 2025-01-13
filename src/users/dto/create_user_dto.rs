use serde::Deserialize;
use validator_derive::Validate;

use crate::shared::role::Role;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserDto {
  #[validate(email)]
  pub email: String,
  #[serde(rename = "userName")]
  #[validate(length(
    max = 1024,
    min = 1,
    message = "Password must have at least 1 characters"
  ))]
  pub user_name: String,
  #[validate(length(
    max = 1024,
    min = 1,
    message = "Password must have at least 1 characters"
  ))]
  pub password: String,
  pub role: Role,
}

#[cfg(test)]
mod tests {
  use super::*;
  use fake::faker::internet::en::SafeEmail;
  use fake::faker::lorem::en::Word;
  use fake::Fake;
  use serde_json::json;
  use validator::Validate;

  #[test]
  fn test_create_user_dto_validation() {
    // Test case: Valid data
    let valid_email: String = SafeEmail().fake();
    let valid_user_name: String = Word().fake();
    let valid_password: String = Word().fake();
    let valid_dto = CreateUserDto {
      email: valid_email.clone(),
      user_name: valid_user_name.clone(),
      password: valid_password.clone(),
      role: Role::Admin,
    };

    assert!(
      valid_dto.validate().is_ok(),
      "Valid DTO should pass validation"
    );

    // Test case: Invalid email
    let invalid_email_dto = CreateUserDto {
      email: "invalid-email".to_string(),
      user_name: valid_user_name.clone(),
      password: valid_password.clone(),
      role: Role::Admin,
    };
    assert!(
      invalid_email_dto.validate().is_err(),
      "DTO with invalid email should fail validation"
    );

    // Test case: Empty user_name
    let empty_user_name_dto = CreateUserDto {
      email: valid_email.clone(),
      user_name: "".to_string(),
      password: valid_password.clone(),
      role: Role::Admin,
    };
    assert!(
      empty_user_name_dto.validate().is_err(),
      "DTO with empty user_name should fail validation"
    );

    // Test case: User_name exceeding max length
    let long_user_name = "a".repeat(1025);
    let long_user_name_dto = CreateUserDto {
      email: valid_email.clone(),
      user_name: long_user_name,
      password: valid_password.clone(),
      role: Role::Admin,
    };
    assert!(
      long_user_name_dto.validate().is_err(),
      "DTO with long user_name should fail validation"
    );

    // Test case: Empty password
    let empty_password_dto = CreateUserDto {
      email: valid_email.clone(),
      user_name: valid_user_name.clone(),
      password: "".to_string(),
      role: Role::Admin,
    };
    assert!(
      empty_password_dto.validate().is_err(),
      "DTO with empty password should fail validation"
    );

    // Test case: Password exceeding max length
    let long_password = "a".repeat(1025);
    let long_password_dto = CreateUserDto {
      email: valid_email.clone(),
      user_name: valid_user_name.clone(),
      password: long_password,
      role: Role::Admin,
    };
    assert!(
      long_password_dto.validate().is_err(),
      "DTO with long password should fail validation"
    );

    // Test case: Invalid role (not possible since Role is enum with predefined variants)
    // If validation logic for role is needed, it can be added in the `Role`.
  }

  #[test]
  fn test_create_user_dto_role_deserialization() {
    use serde_json::json;

    // Helper function to generate a JSON payload for CreateUserDto with a fixed template
    fn create_dto_json(role: &str) -> serde_json::Value {
      json!({
          "email": "test@example.com",
          "userName": "test_user",
          "password": "securepassword",
          "role": role
      })
    }

    // Test case: Valid roles
    let roles = vec![
      ("admin", Role::Admin),
      ("manager", Role::Manager),
      ("driver", Role::Driver),
      ("customer", Role::Customer),
    ];

    for (role_str, role_enum) in roles {
      let valid_json = create_dto_json(role_str);
      let dto: CreateUserDto = serde_json::from_value(valid_json).unwrap();
      assert_eq!(
        dto.role, role_enum,
        "Role should deserialize to the correct enum variant"
      );
    }

    // Test case: Invalid role in DTO
    let invalid_json = create_dto_json("invalid_role");
    let result = serde_json::from_value::<CreateUserDto>(invalid_json);
    assert!(
      result.is_err(),
      "Deserialization should fail for invalid role"
    );
  }
}
