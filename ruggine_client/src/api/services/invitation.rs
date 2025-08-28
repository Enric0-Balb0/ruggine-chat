use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::{
	Invitation, InvitationCreateRequest, InvitationUpdateRequest,
	ApiSuccessResponseInvitationReadDto, ApiSuccessResponseVecInvitationReadDto, InvitationUpdateDto,
	ApiSuccessResponseInvitationUpdateDto,
};

/// Invitation management service
#[derive(Clone)]
pub struct InvitationService {
	http_client: ApiClient,
	storage_service: StorageService,
}

impl InvitationService {
	/// Create new invitation service
	pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
		Self {
			http_client,
			storage_service,
		}
	}

	/// Send a new invitation
	pub async fn send_invitation(&self, request: &InvitationCreateRequest) -> Result<Invitation, AuthError> {
		let response: ApiSuccessResponseInvitationReadDto = self.http_client
			.post(ApiEndpoints::INVITATION_SEND, request)
			.await
			.map_err(AuthError::from)?;
		Ok(Invitation::from(response))
	}

	/// Update invitation status (accept/reject)
	pub async fn update_invitation_status(&self, request: &InvitationUpdateRequest) -> Result<InvitationUpdateDto, AuthError> {
		let response: ApiSuccessResponseInvitationUpdateDto = self.http_client
			.patch(ApiEndpoints::INVITATION_UPDATE, request)
			.await
			.map_err(AuthError::from)?;
		Ok(response.data)
	}

	/// Get all invitations for the authenticated user
	pub async fn get_user_invitations(&self) -> Result<Vec<Invitation>, AuthError> {
		let response: ApiSuccessResponseVecInvitationReadDto = self.http_client
			.get(ApiEndpoints::INVITATION_BY_USER)
			.await
			.map_err(AuthError::from)?;

    Ok(Vec::from(response))
	}

	/// Get a single invitation by id (if belongs to user)
	pub async fn get_invitation_by_id(&self, invitation_id: &str) -> Result<Invitation, AuthError> {
		let response: ApiSuccessResponseInvitationReadDto = self.http_client
			.get(&ApiEndpoints::invitation_by_id(invitation_id))
			.await
			.map_err(AuthError::from)?;
		Ok(Invitation::from(response))
	}
}

impl Default for InvitationService {
	fn default() -> Self {
		let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
		let storage_service = StorageService::new();
		Self::new(http_client, storage_service)
	}
}
