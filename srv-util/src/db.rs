use crate::error::*;
use rusqlite::Connection;
pub use rusqlite::Transaction;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;

pub enum DatabaseType {
    PersistentFile(PathBuf),
    Transient,
}

#[derive(Clone)]
pub struct Database(Arc<Mutex<Connection>>);

impl Database {
    pub async fn new(ty: DatabaseType) -> Result<Self> {
        spawn_blocking(move || {
            let conn = match ty {
                DatabaseType::PersistentFile(path) => Connection::open(path.as_path())?,
                DatabaseType::Transient => Connection::open_in_memory()?,
            };

            Ok(Database(Arc::new(Mutex::new(conn))))
        })
        .await?
    }

    pub fn get_inner(&self) -> &Mutex<Connection> {
        &self.0
    }

    pub async fn do_transaction<
        E: From<Error> + Send + 'static,
        R: Send + 'static,
        F: FnOnce(Transaction) -> StdResult<R, E> + Send + 'static,
    >(
        &self,
        f: F,
    ) -> StdResult<R, E> {
        let conn = self.0.clone();
        spawn_blocking(move || {
            let mut conn = conn.lock().unwrap();
            let ret = f(conn.transaction().map_err(|e| Error::from(e))?);
            ret
        })
        .await
        .map_err(|e| Error::from(e))?
    }
}
