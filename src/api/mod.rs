pub mod finder;
pub mod user;
use actix_web::{web, Scope};

pub fn service() -> Scope {
  web::scope("/api")
    .service(user::service())
    .service(finder::service())
}
