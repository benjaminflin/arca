use crate::env::Environment;
use crate::error::Error;
use crate::file;
use crate::user::User;
use crate::volume::Volume;
use actix_web::{web, HttpResponse};
use serde_derive::{Deserialize, Serialize};

pub async fn open(
    req: &web::HttpRequest,
    env: &web::Data<Environment>,
    user: &User,
) -> Result<HttpResponse, Error> {
    #[derive(Deserialize)]
    struct Params {
        init: Option<bool>,
        target: Option<String>,
        tree: Option<bool>,
    }
    

    #[derive(Serialize)]
    struct Response {
        api: f32,
        cwd: file::File,
        files: Vec<file::File>,
    }

    let params: web::Query<Params> =
        web::Query::from_query(req.query_string()).map_err(|_| Error::InvalidParams)?;

    let _ = params.tree; // TODO: handle tree parameter
    if let Some(true) = &params.init {
        let target = params.target.as_deref().unwrap_or("/");
        let vol = Volume::create_or_find(env, user).await?;
        let files = vol.ls_path(target).await?;
        let cwd = vol.root().await?;
        Ok(HttpResponse::Ok().json(Response {
            api: 2.1, 
            cwd,
            files,
        }))
    } else {
        let target = params.target.as_ref().ok_or(Error::InvalidParams)?;
        let vol = Volume::create_or_find(env, user).await?;
        let files = vol.ls_path(target).await?;
        let cwd = vol.root().await?;
        Ok(HttpResponse::Ok().json(Response {
            api: 2.1, 
            cwd,
            files,
        }))
    }
}
