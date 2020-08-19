use crate::models;
use crate::schema;
use crate::schema::users::dsl::{email, id, users};
use crate::AppData;

use actix_http::ResponseBuilder;
use actix_session::Session;
use actix_web::{error, guard, http, web, HttpResponse, Responder, Scope};
use diesel::prelude::*;
use publicsuffix::List;
use serde_derive::Deserialize;
use std::fmt;

struct UserData {
  list: publicsuffix::List,
}

#[derive(Debug)]
enum UserError {
  InvalidEmail,
  UserAlreadyExists,
  HashError,
  DbError,
  NotFound,
  SessionError,
  UserNotAuthenticated,
  UserNotAuthorized,
}

impl fmt::Display for UserError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use UserError::*;
    match *self {
      InvalidEmail => write!(f, "Invalid Email Address"),
      UserAlreadyExists => write!(f, "User Already Exists"),
      NotFound => write!(f, "User Not Found"),
      UserNotAuthenticated => write!(f, "User Not Authenticated"),
      UserNotAuthorized => write!(f, "User Not Authorized"),
      _ => write!(f, "Internal Server Error"),
    }
  }
}

impl error::ResponseError for UserError {
  fn status_code(&self) -> http::StatusCode {
    use UserError::*;
    match *self {
      InvalidEmail | UserAlreadyExists => http::StatusCode::BAD_REQUEST,
      NotFound => http::StatusCode::NOT_FOUND,
      UserNotAuthorized | UserNotAuthenticated => http::StatusCode::UNAUTHORIZED,
      _ => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
  fn error_response(&self) -> HttpResponse {
    ResponseBuilder::new(self.status_code())
      .set_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
      .body(format!("Error: {}", self))
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
  session: Session,
) -> Result<impl Responder, UserError> {
  // validate email
  data
    .list
    .parse_email(&form.email)
    .map_err(|_| UserError::InvalidEmail)?;
  // create password hash
  // TODO: find out what cost should be used based on benchmarks
  let hash = bcrypt::hash_with_result(form.password.clone(), 10)
    .map_err(|_| UserError::HashError)?
    .to_string();

  let new_user = models::NewUser {
    email: form.email.clone(),
    pass_hash: hash,
  };

  // make db connection
  let conn = app_data.pool.get().map_err(|_| UserError::DbError)?;

  // check if user exists
  if users
    .filter(email.eq(&form.email))
    .first::<models::User>(&conn)
    .is_ok()
  {
    return Err(UserError::UserAlreadyExists);
  }

  // insert into table
  let user = diesel::insert_into(schema::users::table)
    .values(&new_user)
    .returning((id, email))
    .get_result::<models::ClientUser>(&conn)
    .map_err(|_| UserError::DbError)?;

  session
    .set("id", user.id)
    .map_err(|_| UserError::SessionError)?;

  Ok(HttpResponse::Ok().json(user))
}

async fn login(
  form: web::Form<UserFormData>,
  app_data: web::Data<AppData>,
  session: Session,
) -> Result<impl Responder, UserError> {
  // make db connection
  let conn = app_data.pool.get().map_err(|_| UserError::DbError)?;

  // find the user from the id
  let user = users
    .filter(email.eq(&form.email))
    .first::<models::User>(&conn)
    .map_err(|_| UserError::NotFound)?;

  // verify password
  bcrypt::verify(&form.password, &user.pass_hash).map_err(|_| UserError::UserNotAuthenticated)?;

  // construct user for client
  let user = models::ClientUser {
    email: user.email,
    id: user.id,
  };

  // set cookie
  session
    .set("id", user.id)
    .map_err(|_| UserError::SessionError)?;

  Ok(HttpResponse::Ok().json(user))
}

async fn logout(session: Session) -> impl Responder {
  session.purge();
  HttpResponse::Ok()
}

async fn user_info(
  uid: web::Path<(uuid::Uuid,)>,
  app_data: web::Data<AppData>,
) -> Result<impl Responder, UserError> {
  // make db connection
  let conn = app_data.pool.get().map_err(|_| UserError::DbError)?;

  // find the user from the id
  let user = users
    .find(uid.0)
    .select((id, email))
    .first::<models::ClientUser>(&conn)
    .map_err(|_| UserError::NotFound)?;

  Ok(HttpResponse::Ok().json(user))
}

async fn delete_user(
  uid: web::Path<(uuid::Uuid,)>,
  app_data: web::Data<AppData>,
  session: Session,
) -> Result<impl Responder, UserError> {
  // make connection
  let conn = app_data.pool.get().map_err(|_| UserError::DbError)?;

  // get the user
  let user = users
    .find(uid.0)
    .select((id, email))
    .first::<models::ClientUser>(&conn)
    .map_err(|_| UserError::NotFound)?;

  // validate user id
  if let Some(uid) = session
    .get::<uuid::Uuid>("id")
    .map_err(|_| UserError::SessionError)?
  {
    if uid != user.id {
      return Err(UserError::UserNotAuthorized);
    }
  } else {
    return Err(UserError::UserNotAuthenticated);
  }

  // delete the user
  let user = diesel::delete(users.filter(id.eq(uid.0)))
    .returning((id, email))
    .get_result::<models::ClientUser>(&conn)
    .map_err(|_| UserError::DbError)?;

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
    .route(
      "/login",
      web::get()
        .guard(guard::Header(
          "Content-Type",
          "application/x-www-form-urlencoded",
        ))
        .to(login),
    )
    .route("/logout", web::get().to(logout))
    .service(
      web::resource("/{id}")
        .route(web::get().to(user_info))
        .route(web::delete().to(delete_user)),
    )
}
