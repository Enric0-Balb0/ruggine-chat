use async_trait::async_trait;
use sqlx::Error as SqlxError;
use mockall::automock;

use crate::entity::group_membership::{NewGroupMembership};
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

#[async_trait]
#[automock]
pub trait GroupMembershipRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_membership: NewGroupMembership) -> Result<i32, SqlxError>;
    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError>;
    async fn find_by_user_id_and_group_id(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError>;
    /* // user_id is passed because the user can see all other memberships in the group
    // only if he is a member of the group
    async fn find_by_group_chat_id(&self, group_chat_id: i32, user_id: i32) -> Result<Vec<GroupMembership>, SqlxError>; */
}
