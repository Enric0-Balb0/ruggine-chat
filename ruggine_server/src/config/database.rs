use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use async_trait::async_trait;
use core::task;
use dashmap::DashMap;
use sqlx::{Either, Error, Executor, PgPool, Pool, Postgres, Transaction};
use std::collections::hash_map::RandomState;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::task_local;

pub type TaskId = u64;

task_local! {
    static CURRENT_TASK_ID: TaskId;
}

pub struct Database {
    pub pool: Pool<Postgres>,
    // mappa condivisa TaskId -> Transaction
    pub tx_map: DashMap<TaskId, Transaction<'static, Postgres>>,
    pub task_counter: AtomicU64,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init(database_url: String) -> Result<Self, Error>
    where
        Self: Sized;

    fn get_pool(&self) -> &Pool<Postgres>;

    async fn with_tx<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&Database) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Send + From<ApiError> + std::fmt::Debug;

    fn get_tx_mut(
        &self,
    ) -> Option<dashmap::mapref::one::RefMut<'_, TaskId, Transaction<'static, Postgres>>>;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn init(database_url: String) -> Result<Self, Error> {
        /* let pool = PgPoolOptions::new()
            .max_connections(200)                             // numero massimo di connessioni nel pool
            .min_connections(50)                              // numero minimo di connessioni già pronte
            .acquire_timeout(std::time::Duration::from_secs(5)) // tempo massimo di attesa per una connessione libera
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");*/
        let pool = PgPool::connect(&database_url).await?;
        Ok(Self {
            pool,
            tx_map: DashMap::new(),
            task_counter: AtomicU64::new(1),
        })
    }

    fn get_pool(&self) -> &Pool<Postgres> {
        if self.get_tx_mut().is_some() {
            error!("get_pool called, but a transaction is active")
        }
        &self.pool
    }

    async fn with_tx<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&Database) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Send + From<ApiError> + std::fmt::Debug,
    {
        info!("Trying to start a new transaction");
        // 0. verify no transaction already open
        if self.get_tx_mut().is_some() {
            error!("Transaction already exists");
            return Err(ApiError::DbError(DbError::SomethingWentWrong(
                "Transaction already open".to_string(),
            ))
            .into());
        }

        // 1. apre la transazione
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return Err(ApiError::DbError(DbError::SomethingWentWrong(
                    "Impossible to create the transaction".to_string(),
                ))
                .into())
            }
        };

        // 1.b: prende un advisory lock legato alla transazione
        let lock_key: i64 = 42; // <-- Chiave per TUTTE le transazioni, sono poche per ora e va bene
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(lock_key)
            .execute(&mut tx)
            .await
            .map_err(|e| {
                error!("Failed to acquire advisory lock: {}", e);
                ApiError::DbError(DbError::SomethingWentWrong(
                    "Impossible to acquire advisory lock".to_string(),
                ))
            })?;
        info!("Acquired advisory lock");

        info!("Started transaction");
        let task_id = self.task_counter.fetch_add(1, Ordering::SeqCst);

        self.tx_map.insert(task_id, tx);

        // 2. esegue la closure nel contesto del task-local
        info!("Started transaction task");
        let result = CURRENT_TASK_ID
            .scope(task_id, async { f(self).await })
            .await;
        info!("Finished transaction task");

        // 3. commit/rollback + cleanup
        match result {
            Ok(val) => {
                if let Some((_, mut tx)) = self.tx_map.remove(&task_id) {
                    if let Err(e) = tx.commit().await {
                        error!("Failed to commit transaction: {:?}", e);
                        return Err(ApiError::DbError(DbError::SomethingWentWrong(
                            "Impossible to commit the transaction".to_string(),
                        ))
                        .into());
                    }
                    info!("Transaction commited successfully");
                }
                Ok(val)
            }
            Err(err) => {
                error!("Failed to complete transaction: {:?}", err);
                if let Some((_, mut tx)) = self.tx_map.remove(&task_id) {
                    let _ = tx.rollback().await;
                }
                Err(err)
            }
        }
    }

    fn get_tx_mut(
        &self,
    ) -> Option<dashmap::mapref::one::RefMut<'_, TaskId, Transaction<'static, Postgres>>> {
        CURRENT_TASK_ID
            .try_with(|task_id| self.tx_map.get_mut(&*task_id))
            .ok()
            .flatten()
    }
}

impl Database {
    pub fn get_conn(&self) -> DbConn<'_> {
        if let Some(tx) = self.get_tx_mut() {
            DbConn::Tx(tx)
        } else {
            DbConn::Pool(&self.pool)
        }
    }
}

use dashmap::mapref::one::RefMut;
use futures::Stream;
use sqlx::postgres::{PgQueryResult, PgRow, PgTypeInfo};
use tracing::{error, info};

pub enum DbConn<'a> {
    Tx(RefMut<'a, TaskId, Transaction<'static, Postgres>>),
    Pool(&'a Pool<Postgres>),
}

/*
impl Database {
    pub async fn create_tx_mut(&self) {
        // 1. apre la transazione
        let tx = self.pool.begin().await.unwrap();

        self.tx_map.insert(42, tx);
    }

    pub fn get_tx_mut_v2(
        &self,
        task_id: TaskId,
    ) -> Option<dashmap::mapref::one::RefMut<'_, TaskId, Transaction<'static, Postgres>>> {
        self.tx_map.get_mut(&task_id)
    }

    pub async fn commit_tx_mut(&self, task_id: TaskId) -> Result<(), Error> {
        // rimuovi la tx dalla mappa fuori da qualsiasi borrow di DashMap
        let tx = self.tx_map.remove(&task_id)
            .map(|(_, tx)| tx); // prendi solo il Transaction

        if let Some(mut tx) = tx {
            tx.commit().await?; // ora è sicuro fare await
        }

        Ok(())
    }
}

 */

#[cfg(test)]
mod database_tests {
    use std::sync::Arc;

    use crate::error::db_error::DbError;
    use crate::{
        config::database::{Database, DatabaseTrait, DbConn},
        error::api_error::ApiError,
    };
    use once_cell::sync::Lazy;
    use sqlx::{Executor, PgPool};

    #[tokio::test]
    async fn test_init_database() {
        dotenv::dotenv().ok();
        let db = Database::init(std::env::var("TEST_DATABASE_URL").unwrap()).await.unwrap();
        let pool: &PgPool = db.get_pool();
        // proviamo una query semplice
        let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn test_with_tx_commit() {
        dotenv::dotenv().ok();
        let db = Database::init(std::env::var("TEST_DATABASE_URL").unwrap()).await.unwrap();

        let pool = db.get_pool().clone();

        let result: Result<i32, ApiError> = db
            .with_tx(|_db| {
                let pool = pool.clone();
                async move {
                    let row: (i32,) = sqlx::query_as("SELECT 42").fetch_one(&pool).await.unwrap();
                    Ok(row.0)
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_tx_rollback() {
        dotenv::dotenv().ok();
        let db = Arc::new(Database::init(std::env::var("TEST_DATABASE_URL").unwrap()).await.unwrap());
        let db_clone = db.clone();

        let result: Result<i32, ApiError> = db
            .with_tx(|_db| async move {
                let conn = db_clone.get_conn();
                match conn {
                    DbConn::Tx(mut tx) => {
                        let _ = sqlx::query("CREATE TEMP TABLE tmp(x int)")
                            .execute(&mut *tx)
                            .await
                            .unwrap();
                        // simuliamo errore -> rollback
                        Err(ApiError::DbError(DbError::SomethingWentWrong(
                            "Forced".to_string(),
                        )))
                    }
                    _ => panic!("There should be a transaction"),
                }
            })
            .await;

        assert!(result.is_err());

        // Verifica che la tabella temp non esista più (rollback)
        let pool = db.get_pool();
        let err = sqlx::query("SELECT * FROM tmp")
            .execute(pool)
            .await
            .unwrap_err();
        
        let error_string = err.to_string();
        assert!(
            error_string.contains("relation \"tmp\" does not exist") || 
            error_string.contains("does not exist") ||
            error_string.contains("table \"tmp\" does not exist") ||
            error_string.contains("la relazione \"tmp\" non esiste") ||
            error_string.contains("non esiste"),
            "Expected table not to exist, but got error: {}", error_string
        );
    }

    #[tokio::test]
    async fn test_transaction_already_open() {
        dotenv::dotenv().ok();
        let db = Arc::new(Database::init(std::env::var("TEST_DATABASE_URL").unwrap()).await.unwrap());
        let db_clone = db.clone();

        let result = db
            .with_tx(|_db| async move {
                // apriamo una seconda transazione dentro la prima
                let nested = db_clone.with_tx(|_| async { Ok::<i32, ApiError>(123) }).await;
                assert!(nested.is_err());
                Ok::<i32, ApiError>(1)
            })
            .await;

        assert_eq!(result.unwrap(), 1);
    }
}
