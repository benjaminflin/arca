use diesel::{self, Queryable};

use diesel::sql_types::Uuid;

#[derive(Queryable)]
pub struct User {
  pub id: Uuid,
  pub email: String,
  pub pass_hash: String,
}
