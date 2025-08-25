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
        // Check if the group exists first
        self.group_chat_service
            .find_by_id(group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                    ApiError::TextMessageError(TextMessageError::GroupChatNotFound)
                }
                _ => e,
            })?;

        // Check if the auth_user has membership in the group
        let membership = self.group_membership_service
            .find_active_by_user_id_and_group_id(auth_user_id, group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                    ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)
                }
                _ => e,
            })?;

        // Check if it has an active membership
        if membership.membership_status != MembershipStatus::Active {
            return Err(ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages));
        }

        // First get the last sent message in order not to update already sent messages
        let payload = TextMessageLastSentAtDto {
            group_chat_id,
        };
        let mut last_sent_message_datetime: DateTime<Utc> = Utc.timestamp_opt(0, 0).unwrap();
        let last_sent_info = self.find_info_last_sent_at(auth_user_id, payload).await?;
        if last_sent_info.is_some() {
            last_sent_message_datetime = self.find_by_id(last_sent_info.unwrap().text_message_id).await?.sent_at;
        }

        let messages = match self.text_message_repo
            .find_by_group_chat_id_datetime_range(group_chat_id, last_sent_message_datetime, Utc::now())
            .await {
                Ok(data) => {
                    let mut valid_messages = vec![];

                    for message in &data {
                        let payload = TextMessageInfoSentAtDtoUpdate {
                            text_message_id: message.id,
                            sent_at: Utc::now(),
                        };

                        match self.update_sent_at(auth_user_id, payload).await {
                            Ok(_) => valid_messages.push(message.clone()),
                            Err(e) => match e {
                                ApiError::TextMessageError(TextMessageError::CannotUpdateSentAtAgain) => {
                                    // It can happen if many tasks retrieve messages
                                    warn!("Tried to set sent at, but it was already set");
                                },
                                other => return Err(ApiError::DbError(DbError::SomethingWentWrong(other.to_string()))),
                            },
                        }
                    }

                    valid_messages
                },
                Err(e) => {
                    return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())));
                }
            };
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
