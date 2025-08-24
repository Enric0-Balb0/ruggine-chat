use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageSentAtDtoUpdate};
use crate::entity::group_membership::MembershipStatus;
use crate::entity::text_message::TextMessageInfoUpdate;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn update_sent_at_internal(&self, auth_user_id: i32, payload: TextMessageSentAtDtoUpdate) -> Result<TextMessageInfoReadDto, ApiError> {
        let text_message_info = self.find_info_by_user_id_and_message_id(auth_user_id, payload.text_message_id).await?;

        let info = self.find_info_by_id(text_message_info.id).await?;
        if info.sent_at.is_some() {
            return Err(ApiError::TextMessageError(TextMessageError::CannotUpdateSentAtAgain));
        }

        // Update sent at
        let text_message_dto_update = TextMessageInfoUpdate {
            id: text_message_info.id,
            sent_at: Some(payload.sent_at),
            read_at: None,
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