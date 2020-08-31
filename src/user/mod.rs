pub mod error;

use super::schema::users;
use super::volume::*;
use diesel::prelude::*;
use diesel::{pg::PgConnection, Insertable, Queryable};
use error::Result;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
pub struct User {
    pub(crate) id: uuid::Uuid,
    pub(crate) email: String,
    #[serde(skip)]
    pub(crate) pass_hash: String,
    pub(crate) volumes: Option<Vec<Volume>>,
}

#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    email: String,
    pass_hash: String,
    volumes: Option<Vec<Volume>>,
}


impl User {
    pub fn create(
        conn: &PgConnection,
        email: &str,
        password: &str,
        volumes: Option<Vec<Volume>>,
    ) -> Result<Self> {
        let pass_hash = bcrypt::hash_with_result(password, 10)?.to_string();

        let new_user = NewUser {
            email: email.to_owned(),
            pass_hash,
            volumes,
        };

        diesel::insert_into(super::schema::users::table)
            .values(&new_user)
            .get_result::<Self>(conn)
            .map_err(Into::into)
    }

    pub fn find(conn: &PgConnection, email: &str) -> Result<Self> {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users;

        users
            .filter(dsl_email.eq(&email))
            .first::<Self>(conn)
            .map_err(Into::into)
    }

    pub fn find_by_id(conn: &PgConnection, id: uuid::Uuid) -> Result<Self> {
        use crate::schema::users::dsl::users;

        users
            .find(id)
            .first::<Self>(conn)
            .map_err(Into::into)
    }

    pub fn update(self, conn: &PgConnection) -> Result<Self> {
        use crate::schema::users::dsl::id;
        use crate::schema::users::dsl::users;

        diesel::update(users.filter(id.eq(self.id)))
            .set(&self)
            .get_result::<Self>(conn)
            .map_err(Into::into)
    }

    pub fn delete(self, conn: &PgConnection) -> Result<Self> {
        use crate::schema::users::dsl::id;
        use crate::schema::users::dsl::users;

        diesel::delete(users.filter(id.eq(self.id)))
            .get_result::<Self>(conn)
            .map_err(Into::into)
    }
}
