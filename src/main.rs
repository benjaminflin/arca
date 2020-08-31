#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

mod api;
mod file;
mod volume;
mod user;
mod env;
mod error;

use crate::api::finder;
use actix_files::NamedFile;
use actix_http::cookie::SameSite;
use actix_session::CookieSession;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::{HttpRequest, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use dotenv::dotenv;
use std::path::PathBuf;
use env::Environment;


async fn app(req: HttpRequest) -> Result<NamedFile> {
    let dist_dir = "app/dist";
    let filename = req.match_info().query("filename");
    let path: PathBuf = if filename.len() == 0 {
        [dist_dir, "index.html"].iter().collect()
    } else {
        [dist_dir, filename].iter().collect()
    };
    Ok(NamedFile::open(path)?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("Canno find DATABASE_URL in .env");
    let addr = std::env::var("BIND_ADDR").expect("Cannot find BIND_ADDR in .env");

    println!("Connecting to database {}", database_url);

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");

    let root = finder::init();
    println!("Starting actix server on {}", &addr);
    
    HttpServer::new({
        let addr = addr.clone();
        move || {
        App::new()
            // '/' -> '/app'
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict),
            )
            .route(
                "/",
                web::get().to(|| {
                    HttpResponse::PermanentRedirect()
                        .header("Location", "/app")
                        .finish()
                }),
            )
            .service(
                web::scope("/app")
                    .route("/{filename:.*}", web::get().to(app))
                    .default_service(web::get().to(app)),
            )
            .service(api::service())
            .data(Environment {
                db_pool: pool.clone(),
                finder_root: root.clone(),
                bind_addr: addr.clone(),
            })
    }})
    .bind(addr)?
    .run()
    .await
}
