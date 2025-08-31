use tracing::debug;

use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageLastSentAtDto, TextMessageReadDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn find_first_message_with_no_sent_at_inner(&self, auth_user_id: i32, group_chat_id: i32) -> Result<Option<TextMessageReadDto>, ApiError> {
        // Check if group exists
        self.group_chat_service
            .find_by_id(group_chat_id)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                    ApiError::TextMessageError(TextMessageError::GroupChatNotFound)
                }
                other => other,
            })?;

        // Check if he is an active member
        let membership = self.group_membership_service
            .find_active_by_user_id_and_group_id(auth_user_id, group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                    ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)
                }
                _ => e,
            })?;
        if membership.membership_status != MembershipStatus::Active {
            return Err(ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages));
        }

        // Search the last sent at message for the user in the group
        let text_message = self.text_message_repo
            .find_first_message_with_no_sent_at(auth_user_id, group_chat_id)
            .await
            .map_err(|e| ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))?;

        match text_message {
            Some(ref text_message) => {
                // fai qualcosa con text_message
            },
            None => {
                // nessun record trovato
                debug!("Not found first message with no sent for user {}", auth_user_id);
            }
        }

        if text_message.is_none() {
            return Ok(None);
        }

        Ok(Some(TextMessageReadDto::from(text_message.unwrap())))
    }
}