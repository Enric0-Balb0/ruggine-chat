use chrono::Utc;
use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageInfoSentAtDtoUpdate};
use crate::entity::group_membership::MembershipStatus;
use crate::entity::text_message::TextMessageInfoUpdate;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn update_sent_at_internal(&self, auth_user_id: i32, payload: TextMessageInfoSentAtDtoUpdate) -> Result<TextMessageInfoReadDto, ApiError> {
        let info = self.find_info_by_user_id_and_message_id(auth_user_id, payload.text_message_id).await?;

        let message = match self.find_by_id(payload.text_message_id).await {
            Ok(message) => message,
            Err(e) => return Err(e),
        };

        if info.sent_at.is_some() {
            return Err(ApiError::TextMessageError(TextMessageError::CannotUpdateSentAtAgain));
        }

        if !payload.sent_at.clone().le(&Utc::now()) || !payload.sent_at.clone().ge(&message.sent_at) {
            return Err(ApiError::TextMessageError(TextMessageError::SentAtMustBeLessOrEqualsToNowAndGreaterThanMessageCreation));
        }

        // Update sent at
        let text_message_dto_update = TextMessageInfoUpdate {
            id: info.id,
            sent_at: Some(payload.sent_at),
            read_at: None,
        };
        match self.text_message_repo.update_info(text_message_dto_update).await {
            Ok(_) => {
                self.find_info_by_id(info.id).await
            },
            Err(e) => {
                Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))
            }
        }
    }
}