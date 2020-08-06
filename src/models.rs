use super::schema::users;
use diesel::{self, Queryable};
use serde_derive::Serialize;
#[derive(Queryable)]
pub struct User {
  pub id: uuid::Uuid,
  pub email: String,
  pub pass_hash: String,
  pub os_user: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct ClientUser {
  pub id: uuid::Uuid,
  pub email: String,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
  pub email: String,
  pub pass_hash: String,
}
