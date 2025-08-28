use sqlx::Error;
use tracing::info;
use crate::config::database::DatabaseTrait;
use crate::repository::user_repository::UserRepository;

impl UserRepository {
    pub async fn update_online_inner(&self, user_id: i32, is_online: bool) -> Result<(), Error> {
        let query = sqlx::query(
            r#"
            UPDATE "user"
            SET is_online = $1, updated_at = now()
            WHERE id = $2
            "#
        )
            .bind(is_online)
            .bind(user_id);

        let result = if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.execute(&mut *tx_ref).await?
        } else {
            query.execute(self.db_conn.get_pool()).await?
        };

        if result.rows_affected() == 0 {
            info!("set_user_online: no user found with id {}", user_id);
        } else {
            info!("set_user_online: user {} is_online = {}", user_id, is_online);
        }

        Ok(())
    }
}