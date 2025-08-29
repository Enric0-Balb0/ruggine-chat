use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Error as SqlxError, Error};
use mockall::automock;
use crate::config::database::Database;
use crate::entity::group_membership::{GroupMembership, MembershipStatus, NewGroupMembership, UpdateGroupMembership};
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

#[async_trait]
#[automock]
pub trait GroupMembershipRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_membership: NewGroupMembership) -> Result<i32, SqlxError>;
    async fn update(&self, update_group_membership: UpdateGroupMembership) -> Result<(), SqlxError>;
    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError>;
    async fn find_active_by_user_id_and_group_id(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError>;
    async fn find_by_user_id_and_group_id(&self, user_id: i32, group_id: i32, membership_statuses: Vec<MembershipStatus>) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError>;
    async fn find_by_group_chat_id(&self, group_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError>;
    async fn find_connected_users(
        &self,
        user_id: i32,
    ) -> Result<Vec<i32>, SqlxError>;
    async fn find_connected_users_and_online(
        &self,
        user_id: i32,
    ) -> Result<Vec<i32>, SqlxError>;

    async fn promote_admin_if_none(&self, group_chat_id: i32) -> Result<Option<GroupMembershipWithInvitationRow>, SqlxError>;

    /* // user_id is passed because the user can see all other memberships in the group
    // only if he is a member of the group
    async fn find_by_group_chat_id(&self, group_chat_id: i32, user_id: i32) -> Result<Vec<GroupMembership>, SqlxError>; */
    fn db_conn(&self) -> Arc<Database>;
}
