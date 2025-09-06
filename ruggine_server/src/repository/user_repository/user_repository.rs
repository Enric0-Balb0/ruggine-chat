use crate::config::database::{Database, DatabaseTrait};
use std::sync::Arc;
use sqlx::Error as SqlxError;

#[derive(Clone)]
pub struct UserRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl UserRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_email(&self, email: String) -> Result<i32, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "user" WHERE email = $1"#
        )
            .bind(email)
            .execute(self.db_conn.get_pool())
            .await?;

        Ok(result.rows_affected() as i32)
    }


    pub async fn delete_by_id(&self, id: i32) -> Result<i32, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "user" WHERE id = $1"#
        )
            .bind(id)
            .execute(self.db_conn.get_pool())
            .await?;

        Ok(result.rows_affected() as i32)
    }
}
