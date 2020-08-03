#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::{HttpRequest, Result};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

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
async fn start(addr: String) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // '/' -> '/app'
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
    })
    .bind(addr)?
    .run()
    .await
}

fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let addr = env::var("BIND_ADDR").expect("BIND_ADDR must be set");
    println!("Connecting to database {}", database_url);
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url));
    println!("Starting actix server on {}", addr);
    start(addr)
}
