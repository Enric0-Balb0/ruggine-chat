use async_trait::async_trait;
use sqlx::{Error, MySql, MySqlPool, Pool};

pub struct Database {
    pool: Pool<MySql>,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init(database_url: String) -> Result<Self, Error>
        where
            Self: Sized;
    fn get_pool(&self) -> &Pool<MySql>;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn init(database_url: String) -> Result<Self, Error> {
        let pool = MySqlPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &Pool<MySql> {
        &self.pool
    }
}
