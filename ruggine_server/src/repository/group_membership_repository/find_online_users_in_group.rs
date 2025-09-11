use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_online_users_in_group_inner(
        &self,
        group_id: i32,
    ) -> Result<Vec<i32>, Error> {
        let query = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT DISTINCT u.id
            FROM invitation i
            JOIN group_membership gm ON gm.invitation_id = i.id
            JOIN "user" u ON u.id = i.to_user_id
            WHERE i.group_chat_id = $1
              AND gm.membership_status = 'active'
              AND u.is_online = TRUE
              AND u.user_status = 'active'
            "#
        )
            .bind(group_id);

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_all(&mut *tx_ref).await
        } else {
            query.fetch_all(self.db_conn.get_pool()).await
        }
    }
}
