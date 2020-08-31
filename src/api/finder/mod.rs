pub mod ops;

use crate::user::User;
use crate::user::error::Error as UserError;
use crate::env::Environment;
use actix_session::Session;
use actix_web::{web, HttpResponse, Resource};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use crate::error::Error;

async fn command(
    req: web::HttpRequest,
    env: web::Data<Environment>,
    query: web::Query<HashMap<String, String>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    if let Some(user) = session
        .get::<User>("user")
        .map_err(|_| Error::UserError(UserError::SessionError))?
    {
        let cmd = query.get("cmd").ok_or(Error::InvalidParams)?;

        match &cmd as &str {
            "open" => ops::open(&req, &env, &user).await,
            _ => Ok(HttpResponse::Ok().finish()),
        }
    } else {
        Err(Error::UserError(UserError::NotAuthenticated))
    }
}

pub fn service() -> Resource {
    web::resource("/finder").route(web::to(command))
}

pub fn init() -> PathBuf {
    let root = env::var("FINDER_ROOT").expect("Could not find FINDER_ROOT in .env");
    let root_path = Path::new(&root);
    if !root_path.exists() {
        std::fs::create_dir(root_path)
            .expect(&format!("Could not create finder root directory: {}", root));
    }
    PathBuf::from(root_path)
}
