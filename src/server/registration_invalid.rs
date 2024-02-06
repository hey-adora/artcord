//use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize, Serialize};

pub const MINIMUM_PASSWORD_LENGTH: usize = 10;
pub const BCRYPT_COST: u32 = 12;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
// #[archive(compare(PartialEq), check_bytes)]
// #[archive_attr(derive(Debug))]
pub struct RegistrationInvalidMsg {
    pub general_error: Option<String>,
    pub email_error: Option<String>,
    pub password_error: Option<String>,
}

impl RegistrationInvalidMsg {
    pub fn validate_registration(
        email: &str,
        password: &str,
    ) -> (bool, Option<String>, Option<String>) {
        let email_error = if email.len() < 1 {
            Some("Email field can't be empty.".to_string())
        } else {
            None
        };

        let password_error = if password.len() < MINIMUM_PASSWORD_LENGTH {
            Some("Password field can't be empty.".to_string())
        } else {
            None
        };

        let invalid = email_error.is_some() || password_error.is_some();

        (invalid, email_error, password_error)
    }

    pub fn new() -> Self {
        Self {
            general_error: None,
            email_error: None,
            password_error: None,
        }
    }

    pub fn general(mut self, error: String) -> Self {
        self.general_error = Some(error);

        self
    }
}
