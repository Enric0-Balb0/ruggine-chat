mod create_checked;
mod leave_group;
pub mod group_membership_service;
pub mod group_membership_service_trait;
mod find_by_id_and_user_id;
mod find_by_user_id;
mod find_by_user_id_and_group_id;
mod find_by_group_chat_id_checked;
mod find_by_group_chat_id;
pub mod find_active_by_user_id_and_group_id;
mod find_connected_users;
mod find_connected_users_and_online;

use std::sync::Arc;
use async_trait::async_trait;
use crate::dto::group_membership_dto::{GroupMembershipCreateDto, GroupMembershipReadDto, LeaveGroupMembershipDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
pub use crate::service::group_membership_service::group_membership_service::GroupMembershipService;
pub use crate::service::group_membership_service::group_membership_service_trait::GroupMembershipServiceTrait;
use crate::service::invitation_service::InvitationServiceTrait;

#[async_trait]
impl GroupMembershipServiceTrait for GroupMembershipService {
    fn set_invitation_service(&self, invitation_service: Arc<dyn InvitationServiceTrait>) {
        let mut writable = self.invitation_service.write().unwrap();
        *writable = Some(invitation_service);
    }
    async fn create_checked(&self, payload: GroupMembershipCreateDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        self.create_checked_internal(payload, auth_user_id).await
    }

    async fn leave_group(&self, payload: LeaveGroupMembershipDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        self.leave_group_internal(payload, auth_user_id).await
    }

    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        self.find_by_id_and_user_id_internal(id, user_id).await
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        self.find_by_user_id_internal(user_id).await
    }

    async fn find_active_by_user_id_and_group_id(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        self.find_active_by_user_id_and_group_id_internal(user_id, group_id).await
    }

    async fn find_by_user_id_and_group_id(&self, user_id: i32, group_id: i32, membership_statuses: Vec<MembershipStatus>) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        self.find_by_user_id_and_group_id_internal(user_id, group_id, membership_statuses).await
    }

    async fn find_by_group_chat_id_checked(&self, group_id: i32, auth_user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        self.find_by_group_chat_id_checked_internal(group_id, auth_user_id).await
    }

    async fn find_by_group_chat_id(&self, group_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        self.find_by_group_chat_id_internal(group_id).await
    }

    async fn find_connected_users(&self, auth_user_id: i32) -> Result<Vec<i32>, ApiError> {
        self.find_connected_users_internal(auth_user_id).await
    }

    async fn find_connected_users_and_online(&self, auth_user_id: i32) -> Result<Vec<i32>, ApiError> {
        self.find_connected_users_and_online_internal(auth_user_id).await
    }
}