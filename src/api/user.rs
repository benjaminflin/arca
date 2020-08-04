use crate::models;
use crate::schema;
use crate::schema::users::dsl::{email, id, users};
use crate::AppData;
use actix_http::ResponseBuilder;
use actix_web::{error, guard, http, web, HttpResponse, Responder, Scope};
use diesel::prelude::*;
use publicsuffix::List;
use serde_derive::Deserialize;
use std::fmt;

struct UserData {
  list: publicsuffix::List,
}

#[derive(Debug)]
enum UserCreationError {
  InvalidEmail,
  UserAlreadyExists,
  HashError,
  DbError,
}

impl fmt::Display for UserCreationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use UserCreationError::*;
    match *self {
      InvalidEmail => write!(f, "Invalid Email Address"),
      UserAlreadyExists => write!(f, "User Already Exists"),
      _ => write!(f, "Internal Server Error"),
    }
  }
}

impl error::ResponseError for UserCreationError {
  fn status_code(&self) -> http::StatusCode {
    use UserCreationError::*;
    match *self {
      InvalidEmail | UserAlreadyExists => http::StatusCode::BAD_REQUEST,
      _ => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
  fn error_response(&self) -> HttpResponse {
    ResponseBuilder::new(self.status_code())
      .set_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
      .body(format!("User Creation Error: {}", self))
  }
}

#[derive(Deserialize)]
struct UserFormData {
  email: String,
  password: String,
}

async fn create_user(
  form: web::Form<UserFormData>,
  data: web::Data<UserData>,
  app_data: web::Data<AppData>,
) -> Result<impl Responder, UserCreationError> {
  // validate email
  data
    .list
    .parse_email(&form.email)
    .map_err(|_| UserCreationError::InvalidEmail)?;

  // create password hash
  // TODO: find out what cost should be used based on benchmarks
  let hash = bcrypt::hash_with_result(form.password.clone(), 10)
    .map_err(|_| UserCreationError::HashError)?
    .to_string();

  let new_user = models::NewUser {
    email: form.email.clone(),
    pass_hash: hash,
  };

  // make connection
  let conn = app_data
    .pool
    .get()
    .map_err(|_| UserCreationError::DbError)?;

  // check if user exists
  if users
    .filter(email.eq(&form.email))
    .first::<models::User>(&conn)
    .is_ok()
  {
    return Err(UserCreationError::UserAlreadyExists);
  }

  // insert into table
  let user = diesel::insert_into(schema::users::table)
    .values(&new_user)
    .returning((id, email))
    .get_result::<models::ClientUser>(&conn)
    .map_err(|_| UserCreationError::DbError)?;
  Ok(HttpResponse::Ok().json(user))
}

pub fn service() -> Scope {
  web::scope("/user")
    .data(UserData {
      list: List::fetch().expect("Could not fetch public suffix list"),
    })
    .route(
      "/create",
      web::post()
        .guard(guard::Header(
          "Content-Type",
          "application/x-www-form-urlencoded",
        ))
        .to(create_user),
    )
}
