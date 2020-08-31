use std::path::PathBuf;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::pg::PgConnection;

type DbPool = Pool<ConnectionManager<PgConnection>>;
pub struct Environment {
    pub(crate) db_pool: DbPool,
    pub(crate) finder_root: PathBuf,
    pub(crate) bind_addr: String
}
