use std::fmt;
use actix_http::ResponseBuilder;
use actix_web::{error, http, HttpResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  InvalidEmail,
  AlreadyExists,
  HashError,
  DbError,
  NotFound,
  SessionError,
  NotAuthenticated,
  NotAuthorized,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use Error::*;
    match *self {
      InvalidEmail => write!(f, "Invalid Email Address"),
      AlreadyExists => write!(f, "User Already Exists"),
      NotFound => write!(f, "User Not Found"),
      NotAuthenticated => write!(f, "User Not Authenticated"),
      NotAuthorized => write!(f, "User Not Authorized"),
      _ => write!(f, "Internal Server Error"),
    }
  }
}

impl error::ResponseError for Error {
  fn status_code(&self) -> http::StatusCode {
    use Error::*;
    match *self {
      InvalidEmail | AlreadyExists => http::StatusCode::BAD_REQUEST,
      NotFound => http::StatusCode::NOT_FOUND,
      NotAuthorized | NotAuthenticated => http::StatusCode::UNAUTHORIZED,
      _ => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
  fn error_response(&self) -> HttpResponse {
    ResponseBuilder::new(self.status_code())
      .set_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
      .body(format!("Error: {}", self))
  }
}

impl From<diesel::result::Error> for Error {
    fn from(_err: diesel::result::Error) -> Self {
        Self::DbError
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(_err: bcrypt::BcryptError) -> Self {
        Self::DbError
    }
}
