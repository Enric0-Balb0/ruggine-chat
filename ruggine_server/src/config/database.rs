use async_trait::async_trait;
use sqlx::{Error, PgPool, Pool, Postgres};

pub struct Database {
    pub pool: Pool<Postgres>,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init(database_url: String) -> Result<Self, Error>
    where
        Self: Sized;
        
    fn get_pool(&self) -> &Pool<Postgres>;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn init(database_url: String) -> Result<Self, Error> {
        let pool = PgPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
