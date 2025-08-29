use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::{GroupMembershipRepository};

impl GroupMembershipRepository {
    pub async fn promote_admin_if_none_inner(&self, group_chat_id: i32) -> Result<Option<GroupMembershipWithInvitationRow>, Error> {
        let query = sqlx::query_as::<_, GroupMembershipWithInvitationRow>(
            r#"
            WITH no_admin AS (
                SELECT NOT EXISTS (
                    SELECT 1
                    FROM group_membership gm
                    JOIN invitation i ON gm.invitation_id = i.id
                    WHERE i.group_chat_id = $1
                      AND gm.membership_status = 'active'
                      AND gm.role = 'admin'
                ) AS need_admin
            ),
            candidate AS (
                SELECT gm2.id
                FROM group_membership gm2
                JOIN invitation i2 ON gm2.invitation_id = i2.id
                CROSS JOIN no_admin
                WHERE i2.group_chat_id = $1
                  AND gm2.membership_status = 'active'
                  AND no_admin.need_admin = TRUE
                LIMIT 1
            )
            UPDATE group_membership gm
            SET role = 'admin'
            WHERE gm.id = (SELECT id FROM candidate)
            RETURNING
                gm.id,
                gm.role,
                gm.joined_at,
                gm.left_at,
                gm.membership_status,
                gm.invitation_id,
                (SELECT i.to_user_id FROM invitation i WHERE i.id = gm.invitation_id) AS user_id,
                (SELECT i.group_chat_id FROM invitation i WHERE i.id = gm.invitation_id) AS group_chat_id,
                gm.current_action;
            "#
        )
            .bind(group_chat_id);

        // se c'è una transazione, usala; altrimenti usa la pool
        let response;

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            println!("Using transaction");
            response = query.fetch_optional(&mut *tx_ref).await
        } else {
            println!("Using pool");
            response = query.fetch_optional(self.db_conn.get_pool()).await
        }

        match response {
            Ok(id) => Ok(id),
            Err(err) => {
                eprintln!("Failed to insert group membership: {:?}", err);
                Err(err)
            }
        }

    }
}