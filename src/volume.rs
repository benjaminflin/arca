use std::path::{Path, PathBuf};
use std::io::Write;
use serde_derive::{Deserialize, Serialize};
use diesel::{Queryable, sql_types::Text, deserialize::{self, FromSql}, serialize::{self, Output, ToSql}, backend::Backend};
use super::error::Result;
use super::env::Environment;
use super::user::User;
use super::file::File;

#[derive(Debug, Deserialize, Serialize, Queryable)]
#[repr(transparent)]
pub struct Volume {
    pub(crate) path: PathBuf
}

impl<DB> FromSql<Text, DB> for Volume
where DB: Backend,
      String: FromSql<Text, DB> {
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        Ok(Self {
            path: String::from_sql(bytes)?.into()
        })
    }
}

impl<DB> ToSql<Text, DB> for Volume
where DB: Backend,
      String: ToSql<Text, DB> {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.path
            .canonicalize()?
            .to_string_lossy()
            .into_owned()
            .to_sql(out)
    }
}


impl Volume {
    
    pub async fn create_or_find(env: &Environment, user: &User) -> Result<Self> {
        let path = [AsRef::<Path>::as_ref(&env.finder_root), user.id.to_simple().to_string().as_ref()]
           .iter()
           .collect::<PathBuf>();

        if !path.exists() {
            tokio::fs::create_dir(&path).await?;
        }

        Ok(Self {
            path
        })
    }

    pub async fn root(&self) -> Result<File> {
        File::info(self, "/").await
    }
    pub async fn ls(&self) -> Result<Vec<File>> {
        File::open_dir(self, "/").await
    }

    pub async fn ls_path(&self, path: impl AsRef<Path>) -> Result<Vec<File>> {
        File::open_dir(self, path.as_ref()).await
    }
}
