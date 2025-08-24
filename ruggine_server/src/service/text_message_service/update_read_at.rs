use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageReadAtDtoUpdate};
use crate::entity::group_membership::MembershipStatus;
use crate::entity::text_message::TextMessageInfoUpdate;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn update_read_at_internal(&self, auth_user_id: i32, payload: TextMessageReadAtDtoUpdate) -> Result<TextMessageInfoReadDto, ApiError> {
        let text_message_info = self.find_info_by_user_id_and_message_id(auth_user_id, payload.text_message_id).await?;

        let info = self.find_info_by_id(text_message_info.id).await?;
        if info.read_at.is_some() {
            return Err(ApiError::TextMessageError(TextMessageError::CannotUpdateReadAtAgain));
        }

        if info.sent_at.is_none() {
            return Err(ApiError::TextMessageError(TextMessageError::CannotSetReadAtBeforeSentAt));
        }

        if payload.read_at.clone().lt(&info.sent_at.unwrap()) {
            return Err(ApiError::TextMessageError(TextMessageError::ReadAtMustBeGreaterOrEqualsToSentAt))
        }

        // Update read at
        let text_message_dto_update = TextMessageInfoUpdate {
            id: text_message_info.id,
            sent_at: None,
            read_at: Some(payload.read_at),
        };
        match self.text_message_repo.update_info(text_message_dto_update).await {
            Ok(_) => {
                self.find_info_by_id(text_message_info.id).await
            },
            Err(e) => {
                Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))
            }
        }
    }
}