use libsql::{Builder, params::IntoParams};

use crate::{
    AppError,
    storage::{self, AppStorage},
};

#[derive(Debug)]
pub struct LibSQLStorage {
    path: &'static str,
}

impl LibSQLStorage {
    pub fn new(path: &'static str) -> Self {
        Self { path }
    }
}

impl AppStorage for LibSQLStorage {
    type DBConnectionObject = libsql::Connection;

    async fn init(&mut self) -> Result<Self::DBConnectionObject, crate::AppError> {
        let db = Builder::new_local(self.path)
            .build()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;
        let conn = db.connect().map_err(|e| AppError::AppError(Box::new(e)))?;

        conn.execute("SELECT 1", ())
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        Ok(conn.clone())
    }
}

// example
// let mut db = storage::libsql::LibSQLStorage::new(":memory:");
// let db: libsql::Connection = db.init().await?;
