use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessage;
use crate::repository::text_message_repository::TextMessageRepository;
use sqlx::Error;

impl TextMessageRepository {
    /// Returns the oldest message not read yet to the user in the group chat if available
    pub async fn find_first_message_with_no_read_at_inner(&self, user_id: i32, group_chat_id: i32) -> Result<Option<TextMessage>, Error> {
        let rec = sqlx::query_as::<_, TextMessage>(
            r#"
            SELECT tm.id,
                tm.content,
                tm.sender_id,
                tm.group_chat_id,
                tm.sent_at
            FROM text_message_info tmi
            INNER JOIN text_message tm
                ON tmi.text_message_id = tm.id
            WHERE tmi.user_id = $1
            AND tm.group_chat_id = $2
            AND tmi.read_at IS NULL
            ORDER BY tm.sent_at ASC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .bind(group_chat_id)
        .fetch_optional(self.db_conn.get_pool())
        .await?;


        Ok(rec)
    }
}