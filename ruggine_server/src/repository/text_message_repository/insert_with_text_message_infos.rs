use sqlx::Error;
use tracing::info;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::NewTextMessage;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn insert_with_text_message_infos_inner(&self, new_text_message: NewTextMessage) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let query = sqlx::query_scalar(
            r#"
            WITH inserted_message AS (
                INSERT INTO text_message (content, sender_id, group_chat_id, sent_at)
                VALUES ($1, $2, $3, $4)
                RETURNING id, group_chat_id
            )
            INSERT INTO text_message_info (user_id, text_message_id)
            SELECT i.to_user_id, im.id
            FROM group_membership gm
            JOIN invitation i ON gm.invitation_id = i.id
            JOIN inserted_message im ON i.group_chat_id = im.group_chat_id
            WHERE gm.membership_status = 'active'
            RETURNING text_message_id;
            "#
        )
            .bind(new_text_message.content)
            .bind(new_text_message.sender_id)
            .bind(new_text_message.group_chat_id)
            .bind(now);

        // se c'è una transazione, usala; altrimenti usa la pool
        let response;

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            info!("Using transaction");
            response = query.fetch_one(&mut *tx_ref).await
        } else {
            info!("Using pool");
            response = query.fetch_one(self.db_conn.get_pool()).await
        }

        match response {
            Ok(id) => Ok(id),
            Err(err) => {
                eprintln!("Failed to insert text message repository: {:?}", err);
                Err(err)
            }
        }
    }
}