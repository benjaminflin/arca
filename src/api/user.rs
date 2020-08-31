use crate::env::Environment;
use crate::user::error::{Error, Result};
use crate::user::User;
use actix_session::Session;
use actix_web::{guard, web, HttpResponse, Responder, Scope};
use publicsuffix::List;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct UserFormData {
    email: String,
    password: String,
}

async fn create_user(
    form: web::Form<UserFormData>,
    list: web::Data<publicsuffix::List>,
    env: web::Data<Environment>,
    session: Session,
) -> Result<impl Responder> {
    // validate email
    list.parse_email(&form.email)
        .map_err(|_| Error::InvalidEmail)?;

    let conn = env.db_pool.get().map_err(|_| Error::DbError)?;

    // check if user exists
    User::find(&conn, &form.email)
        .err()
        .ok_or(Error::AlreadyExists)?;

    let user = User::create(&conn, &form.email, &form.password, None)?;

    session
        .set("user", &user)
        .map_err(|_| Error::SessionError)?;

    Ok(HttpResponse::Ok().json(user))
}

async fn login(
    form: web::Form<UserFormData>,
    env: web::Data<Environment>,
    session: Session,
) -> Result<impl Responder> {
    // make db connection
    let conn = env.db_pool.get().map_err(|_| Error::DbError)?;

    // find the user from the id
    let user = User::find(&conn, &form.email)?;

    // verify password
    bcrypt::verify(&form.password, &user.pass_hash).map_err(|_| Error::NotAuthenticated)?;

    // set cookie
    session
        .set("user", &user)
        .map_err(|_| Error::SessionError)?;

    Ok(HttpResponse::Ok().json(user))
}

async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok()
}

async fn user_info(
    uid: web::Path<(uuid::Uuid,)>,
    env: web::Data<Environment>,
) -> Result<impl Responder> {
    // make db connection
    let conn = env.db_pool.get().map_err(|_| Error::DbError)?;

    // find the user from the id
    let user = User::find_by_id(&conn, uid.0)?;
    
    Ok(HttpResponse::Ok().json(user))
}

async fn delete_user(
    uid: web::Path<(uuid::Uuid,)>,
    env: web::Data<Environment>,
    session: Session,
) -> Result<impl Responder> {
    // make connection
    let conn = env.db_pool.get().map_err(|_| Error::DbError)?;

    // get the user
    let user = User::find_by_id(&conn, uid.0)?;
    
    // validate user id
    if let Some(session_user) = session
        .get::<User>("user")
        .map_err(|_| Error::SessionError)?
    {
        if session_user.id != user.id {
            return Err(Error::NotAuthorized);
        }
    } else {
        return Err(Error::NotAuthenticated);
    }

    // delete the user
    let user = user.delete(&conn)?; 

    Ok(HttpResponse::Ok().json(user))
}

pub fn service() -> Scope {
    web::scope("/user")
        .data(List::fetch().expect("Could not fetch public suffix list"))
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
