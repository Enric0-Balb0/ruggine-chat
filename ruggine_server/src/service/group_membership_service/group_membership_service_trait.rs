use std::sync::Arc;
use crate::dto::group_membership_dto::{GroupMembershipCreateDto, GroupMembershipReadDto, LeaveGroupMembershipDto};
use crate::error::api_error::ApiError;
use async_trait::async_trait;
use mockall::automock;
use crate::entity::group_membership::MembershipStatus;
use crate::service::invitation_service::InvitationServiceTrait;

#[async_trait]
#[automock]
pub trait GroupMembershipServiceTrait: Send + Sync {
    fn set_invitation_service(&self, invitation_service: Arc<dyn InvitationServiceTrait>);
    async fn create_checked(&self, payload: GroupMembershipCreateDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError>;
    async fn leave_group(&self, payload: LeaveGroupMembershipDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError>;
    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<GroupMembershipReadDto, ApiError>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError>;
    async fn find_by_user_id_and_group_id(&self, user_id: i32, group_id: i32, membership_statuses: Vec<MembershipStatus>) -> Result<Vec<GroupMembershipReadDto>, ApiError>;
    async fn find_active_by_user_id_and_group_id(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipReadDto, ApiError>;
    async fn find_by_group_chat_id_checked(&self, group_id: i32, auth_user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError>;
    async fn find_by_group_chat_id(&self, group_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError>;
    async fn find_connected_users(&self, auth_user_id: i32) -> Result<Vec<i32>, ApiError>;

}
