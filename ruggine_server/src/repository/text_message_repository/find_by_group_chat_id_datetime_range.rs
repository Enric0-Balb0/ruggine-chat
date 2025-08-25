use sqlx::Error;
use chrono::{DateTime, Utc};
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessage;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_by_group_chat_id_datetime_range_inner(&self, group_chat_id: i32, min_datetime: DateTime<Utc>, max_datetime: DateTime<Utc>) -> Result<Vec<TextMessage>, Error> {
        let messages = sqlx::query_as!(
                    TextMessage,
                    r#"
                    SELECT id, content, sender_id, group_chat_id, sent_at
                    FROM text_message
                    WHERE group_chat_id = $1 AND sent_at >= $2 AND sent_at <= $3
                    ORDER BY sent_at DESC
                    "#,
                    group_chat_id,
                    min_datetime,
                    max_datetime
                )
            .fetch_all(self.db_conn.get_pool())
            .await?;

        Ok(messages)
    }
}
