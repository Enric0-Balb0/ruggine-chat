use async_trait::async_trait;
use sqlx::Error as SqlxError;
use mockall::automock;

use crate::entity::group_membership::{NewGroupMembership, GroupMembership};

#[async_trait]
#[automock]
pub trait GroupMembershipRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_membership: NewGroupMembership) -> Result<i32, SqlxError>;
    async fn find_by_id(&self, id: i32) -> Result<GroupMembership, SqlxError>;
    /* async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembership>, SqlxError>;
    // user_id is passed because the user can see all other memberships in the group
    // only if he is a member of the group
    async fn find_by_group_chat_id(&self, group_chat_id: i32, user_id: i32) -> Result<Vec<GroupMembership>, SqlxError>; */
}
