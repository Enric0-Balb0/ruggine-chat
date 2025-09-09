use chrono::{DateTime, TimeZone, Utc};
use tracing::warn;
use crate::dto::text_message_dto::{TextMessageLastSentAtDto, TextMessageReadDto, TextMessageInfoSentAtDtoUpdate};
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::response::{PaginatedTextMessageResponse, PaginationMetadata};
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn find_messages_not_sent_yet_internal(&self, group_chat_id: i32, auth_user_id: i32) -> Result<PaginatedTextMessageResponse, ApiError> {
        let messages;
        // First get the last sent message in order not to update already sent messages
        let first_to_send = self.find_first_message_with_no_sent_at_inner(auth_user_id, group_chat_id).await?;
        if first_to_send.is_some() {
            let last_sent_message_datetime = self.find_by_id(first_to_send.unwrap().id).await?.sent_at;
            messages = match self.text_message_repo
                .find_by_group_chat_id_datetime_range(group_chat_id, last_sent_message_datetime, Utc::now())
                .await {
                Ok(data) => {
                    let mut valid_messages = vec![];

                    for message in &data {
                        let payload = TextMessageInfoSentAtDtoUpdate {
                            text_message_id: message.id,
                        };

                        match self.update_sent_at(auth_user_id, payload).await {
                            Ok(_) => { },
                            Err(e) => match e {
                                ApiError::TextMessageError(TextMessageError::CannotUpdateSentAtAgain) => {
                                    // It can happen if many tasks retrieve messages
                                    warn!("Tried to set sent at, but it was already set");
                                },
                                ApiError::TextMessageError(TextMessageError::MessageInfoNotFound) => {
                                    // It can happen if user has left the group and then rejoined
                                    warn!("Tried to set sent at, but it was not found");
                                },
                                other => return Err(ApiError::DbError(DbError::SomethingWentWrong(other.to_string()))),
                            },
                        }
                        valid_messages.push(message.clone());
                    }

                    valid_messages
                },
                Err(e) => {
                    return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())));
                }
            };
        }
        else {
            messages = vec![];
        }

        // Get next cursor from the last message's datetime
        let next_cursor = if !messages.is_empty() {
            Some(messages.last().unwrap().sent_at)
        } else {
            None
        };

        // Convert to DTOs
        let data: Vec<TextMessageReadDto> = messages
            .into_iter()
            .map(TextMessageReadDto::from)
            .collect();

        let pagination = PaginationMetadata {
            has_more: true,  // We don't calculate has_more for cursor-based pagination
            next_cursor,
            page_size: data.len(),
            total_count: None, // We don't calculate total count for cursor-based pagination
        };

        Ok(PaginatedTextMessageResponse {
            data,
            pagination,
        })
    }
}
