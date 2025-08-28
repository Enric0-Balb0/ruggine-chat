use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_connected_users_inner(
        &self,
        user_id: i32,
    ) -> Result<Vec<i32>, Error> {
        let query = sqlx::query_scalar::<_, i32>(
            r#"
        SELECT DISTINCT i2.to_user_id
        FROM invitation i1
        JOIN group_membership gm1 ON gm1.invitation_id = i1.id
        JOIN invitation i2 ON i1.group_chat_id = i2.group_chat_id
        JOIN group_membership gm2 ON gm2.invitation_id = i2.id
        WHERE i1.to_user_id = $1
          AND gm1.membership_status = 'active'
          AND gm2.membership_status = 'active'
          AND i2.to_user_id != $1
        "#
        )
            .bind(user_id);

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_all(&mut *tx_ref).await
        } else {
            query.fetch_all(self.db_conn.get_pool()).await
        }
    }
}


