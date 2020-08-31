use super::user;
use actix_http::http;
use actix_web::{error, HttpResponse};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UserError(user::error::Error),
    IoError(tokio::io::Error),
    PathError,
    InvalidParams,
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            UserError(ref e) => write!(f, "{}", e),
            IoError(_) => write!(f, "IO Error"),
            InvalidParams => write!(f, "Invalid Params"),
            PathError => write!(f, "Path Error"),
            Other(ref s) => write!(f, "Internal Error: {}", s),
        }
    }
}

impl error::ResponseError for Error {
    fn status_code(&self) -> http::StatusCode {
        use Error::*;
        match *self {
            InvalidParams => http::StatusCode::BAD_REQUEST,
            UserError(ref e) => e.status_code(),
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(format!("{{ error: {} }}", self))
    }
}
impl From<tokio::io::Error> for Error {
    fn from(e: tokio::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<user::error::Error> for Error {
    fn from(e: user::error::Error) -> Self {
        Self::UserError(e)
    }
}

