use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfoUpdate;
use crate::repository::text_message_repository::TextMessageRepository;
use sqlx::Error;
use tracing::error;

impl TextMessageRepository {
    pub async fn update_read_at_info_inner(&self, text_message_info_id: i32) -> Result<(), Error> {
        let result = sqlx::query(
            "UPDATE text_message_info SET read_at = CURRENT_TIMESTAMP WHERE id = $1"
        )
            .bind(text_message_info_id)
            .execute(self.db_conn.get_pool())
            .await?;

        if result.rows_affected() == 0 {
            error!("Failed to update read at text message info with id {}", text_message_info_id);
            Err(Error::RowNotFound)
        } else {
            Ok(())
        }
    }
}