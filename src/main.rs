#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

mod api;

use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::{HttpRequest, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

struct AppData {
    pool: DbPool,
}

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

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let addr = env::var("BIND_ADDR").expect("BIND_ADDR must be set");

    println!("Connecting to database {}", database_url);

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");
    println!("Starting actix server on {}", addr);

    HttpServer::new(move || {
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
            .service(api::service())
            .data(AppData { pool: pool.clone() })
    })
    .bind(addr)?
    .run()
    .await
}
