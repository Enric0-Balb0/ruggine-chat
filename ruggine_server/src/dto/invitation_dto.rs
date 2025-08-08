use serde::{Deserialize, Serialize};
use utoipa::{openapi::schema, ToSchema};
use validator::Validate;
use chrono::{DateTime, Utc};
use crate::entity::invitation::{Invitation, InvitationStatus};
use serde_json::json;
use crate::entity::group_membership::MemberRole;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
pub struct InvitationCreateDto {
    #[validate(range(min = 1, message = "User ID must be positive"))]
    #[schema(example = 2)]
    pub to_user_id: i32,
    #[validate(range(min = 1, message = "Group chat ID must be positive"))]
    #[schema(example = 1)]
    pub group_chat_id: i32,
    #[schema(example = "member")]
    pub role_at_join: MemberRole
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct InvitationReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 1)]
    pub from_user_id: i32,
    #[schema(example = 2)]
    pub to_user_id: i32,
    #[schema(example = 1)]
    pub group_chat_id: i32,
    #[schema(example = "pending")]
    pub status: InvitationStatus,
    #[schema(example = "2025-01-28T10:00:00Z")]
    pub sent_at: DateTime<Utc>,
    #[schema()]
    pub responded_at: Option<DateTime<Utc>>,
    #[schema(example = "member")]
    pub role_at_join: MemberRole
}

impl From<Invitation> for InvitationReadDto {
    fn from(invitation: Invitation) -> Self {
        InvitationReadDto {
            id: invitation.id,
            from_user_id: invitation.from_user_id,
            to_user_id: invitation.to_user_id,
            group_chat_id: invitation.group_chat_id,
            status: invitation.status,
            sent_at: invitation.sent_at,
            responded_at: invitation.responded_at,
            role_at_join: invitation.role_at_join
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
#[schema(example = json!({
    "status": "accepted",
    "invitation_id": 4
}))]
pub struct InvitationUpdateStatusDto {
    #[schema(example = "accepted")]
    pub status: InvitationStatus,
    #[schema(example = "1")]
    #[validate(range(min = 1, message = "Invitation ID must be positive"))]
    pub invitation_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "id": 1,
    "status": "accepted",
    "responded_at": "2025-01-28T11:00:00Z",
    "group_membership_id": 42
}))]
pub struct InvitationUpdateResponseDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "accepted")]
    pub status: InvitationStatus,
    #[schema(example = "2025-01-28T11:00:00Z")]
    pub responded_at: DateTime<Utc>,
    #[schema(example = 1)]
    pub group_membership_id: Option<i32>, // Optional field for group membership ID if created
}

impl InvitationUpdateResponseDto {
    pub fn set_group_membership_id(&mut self, group_membership_id: i32) {
        self.group_membership_id = Some(group_membership_id);
    }
}

impl From<Invitation> for InvitationUpdateResponseDto {
    fn from(invitation: Invitation) -> Self {
        InvitationUpdateResponseDto {
            id: invitation.id,
            status: invitation.status,
            responded_at: invitation.responded_at.unwrap_or_else(|| Utc::now()),
            group_membership_id: None, // This field can be set later if needed
        }
    }
}

/// Validation helper to ensure status is only accepted or rejected for updates
impl InvitationUpdateStatusDto {
    pub fn validate_status(&self) -> Result<(), String> {
        match self.status {
            InvitationStatus::Accepted | InvitationStatus::Rejected => Ok(()),
            InvitationStatus::Pending => Err("Cannot update invitation status to pending".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_invitation_create_dto_valid() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.to_user_id, 2);
        assert_eq!(create_dto.group_chat_id, 1);
        assert_eq!(create_dto.role_at_join, MemberRole::Member);
    }

    #[test]
    fn test_invitation_create_dto_zero_to_user_id() {
        let create_dto = InvitationCreateDto {
            to_user_id: 0, // Invalid: must be positive
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("to_user_id"));
    }

    #[test]
    fn test_invitation_create_dto_negative_to_user_id() {
        let create_dto = InvitationCreateDto {
            to_user_id: -1, // Invalid: must be positive
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("to_user_id"));
    }

    #[test]
    fn test_invitation_create_dto_zero_group_chat_id() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 0, // Invalid: must be positive
            role_at_join: MemberRole::Member,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("group_chat_id"));
    }

    #[test]
    fn test_invitation_create_dto_negative_group_chat_id() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: -1, // Invalid: must be positive
            role_at_join: MemberRole::Member,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("group_chat_id"));
    }

    #[test]
    fn test_invitation_create_dto_both_fields_invalid() {
        let create_dto = InvitationCreateDto {
            to_user_id: 0, // Invalid
            group_chat_id: -1, // Invalid
            role_at_join: MemberRole::Member,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("to_user_id"));
        assert!(errors.field_errors().contains_key("group_chat_id"));
    }

    #[test]
    fn test_invitation_create_dto_clone() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        let cloned_dto = create_dto.clone();
        assert_eq!(create_dto.to_user_id, cloned_dto.to_user_id);
        assert_eq!(create_dto.group_chat_id, cloned_dto.group_chat_id);
        assert_eq!(create_dto.role_at_join, cloned_dto.role_at_join);
    }

    #[test]
    fn test_invitation_create_dto_debug_format() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        let debug_string = format!("{:?}", create_dto);
        assert!(debug_string.contains("InvitationCreateDto"));
        assert!(debug_string.contains("2"));
        assert!(debug_string.contains("1"));
        assert!(debug_string.contains("Member"));
    }

    #[test]
    fn test_invitation_create_dto_serialization() {
        let create_dto = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Admin,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&create_dto).unwrap();
        assert!(json.contains("2"));
        assert!(json.contains("1"));
        assert!(json.contains("admin"));

        // Test deserialization from JSON
        let deserialized: InvitationCreateDto = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.to_user_id, create_dto.to_user_id);
        assert_eq!(deserialized.group_chat_id, create_dto.group_chat_id);
        assert_eq!(deserialized.role_at_join, create_dto.role_at_join);
    }

    #[test]
    fn test_invitation_read_dto_creation() {
        let naive_datetime = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let read_dto = InvitationReadDto {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: DateTime::from_naive_utc_and_offset(naive_datetime, Utc),
            responded_at: None,
            role_at_join: MemberRole::Member,
        };

        assert_eq!(read_dto.id, 1);
        assert_eq!(read_dto.from_user_id, 1);
        assert_eq!(read_dto.to_user_id, 2);
        assert_eq!(read_dto.group_chat_id, 1);
        assert_eq!(read_dto.status, InvitationStatus::Pending);
        assert_eq!(
            read_dto.sent_at.date_naive(),
            NaiveDate::from_ymd_opt(2025, 1, 28).unwrap()
        );
        assert_eq!(read_dto.responded_at, None);
        assert_eq!(read_dto.role_at_join, MemberRole::Member);
    }

    #[test]
    fn test_invitation_read_dto_with_response() {
        let sent_datetime = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let responded_datetime = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let read_dto = InvitationReadDto {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Accepted,
            sent_at: DateTime::from_naive_utc_and_offset(sent_datetime, Utc),
            responded_at: Some(DateTime::from_naive_utc_and_offset(responded_datetime, Utc)),
            role_at_join: MemberRole::Member,
        };

        assert_eq!(read_dto.status, InvitationStatus::Accepted);
        assert!(read_dto.responded_at.is_some());
        assert_eq!(
            read_dto.responded_at.unwrap().date_naive(),
            NaiveDate::from_ymd_opt(2025, 1, 28).unwrap()
        );
        assert_eq!(read_dto.role_at_join, MemberRole::Member);
    }

    #[test]
    fn test_invitation_read_dto_serialization() {
        let naive_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let read_dto = InvitationReadDto {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            responded_at: None,
            role_at_join: MemberRole::Member,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&read_dto).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"from_user_id\":1"));
        assert!(json.contains("\"to_user_id\":2"));
        assert!(json.contains("\"group_chat_id\":1"));
        assert!(json.contains("\"status\":\"pending\""));
        assert!(json.contains("2025-01-28"));

        // Test deserialization from JSON
        let deserialized: InvitationReadDto = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, read_dto.id);
        assert_eq!(deserialized.from_user_id, read_dto.from_user_id);
        assert_eq!(deserialized.to_user_id, read_dto.to_user_id);
        assert_eq!(deserialized.group_chat_id, read_dto.group_chat_id);
        assert_eq!(deserialized.status, read_dto.status);
        assert_eq!(deserialized.sent_at, read_dto.sent_at);
        assert_eq!(deserialized.responded_at, read_dto.responded_at);
    }

    #[test]
    fn test_invitation_read_dto_from_invitation() {
        let naive_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let invitation = Invitation {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            responded_at: None,
            role_at_join: MemberRole::Member,
        };

        let read_dto: InvitationReadDto = invitation.clone().into();
        assert_eq!(read_dto.id, invitation.id);
        assert_eq!(read_dto.from_user_id, invitation.from_user_id);
        assert_eq!(read_dto.to_user_id, invitation.to_user_id);
        assert_eq!(read_dto.group_chat_id, invitation.group_chat_id);
        assert_eq!(read_dto.status, invitation.status);
        assert_eq!(read_dto.sent_at, invitation.sent_at);
        assert_eq!(read_dto.responded_at, invitation.responded_at);
        assert_eq!(read_dto.role_at_join, invitation.role_at_join);
    }

    #[test]
    fn test_invitation_update_status_dto_valid_accepted() {
        let update_dto = InvitationUpdateStatusDto {
            status: InvitationStatus::Accepted,
            invitation_id: 1,
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.status, InvitationStatus::Accepted);
        assert!(update_dto.validate_status().is_ok());
    }

    #[test]
    fn test_invitation_update_status_dto_valid_rejected() {
        let update_dto = InvitationUpdateStatusDto {
            status: InvitationStatus::Rejected,
            invitation_id: 1,
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.status, InvitationStatus::Rejected);
        assert!(update_dto.validate_status().is_ok());
    }

    #[test]
    fn test_invitation_update_status_dto_invalid_pending() {
        let update_dto = InvitationUpdateStatusDto {
            status: InvitationStatus::Pending,
            invitation_id: 1,
        };

        assert!(update_dto.validate().is_ok()); // Basic validation passes
        assert!(update_dto.validate_status().is_err()); // Custom validation fails
        
        let error = update_dto.validate_status().unwrap_err();
        assert_eq!(error, "Cannot update invitation status to pending");
    }

    #[test]
    fn test_invitation_update_status_dto_serialization() {
        let update_dto = InvitationUpdateStatusDto {
            status: InvitationStatus::Accepted,
            invitation_id: 1,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&update_dto).unwrap();
        assert!(json.contains("\"status\":\"accepted\""));

        // Test deserialization from JSON
        let deserialized: InvitationUpdateStatusDto = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, update_dto.status);
        assert_eq!(deserialized.invitation_id, update_dto.invitation_id);
    }

    #[test]
    fn test_invitation_update_response_dto_creation() {
        let naive_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let response_dto = InvitationUpdateResponseDto {
            id: 1,
            status: InvitationStatus::Accepted,
            responded_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            group_membership_id: None, // This field can be set later if needed
        };

        assert_eq!(response_dto.id, 1);
        assert_eq!(response_dto.status, InvitationStatus::Accepted);
        assert_eq!(
            response_dto.responded_at.date_naive(),
            NaiveDate::from_ymd_opt(2025, 1, 28).unwrap()
        );
    }

    #[test]
    fn test_invitation_update_response_dto_from_invitation_with_response() {
        let sent_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let responded_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let invitation = Invitation {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Accepted,
            sent_at: DateTime::from_naive_utc_and_offset(sent_dt, Utc),
            responded_at: Some(DateTime::from_naive_utc_and_offset(responded_dt, Utc)),
            role_at_join: MemberRole::Admin,
        };

        let response_dto: InvitationUpdateResponseDto = invitation.clone().into();
        assert_eq!(response_dto.id, invitation.id);
        assert_eq!(response_dto.status, invitation.status);
        assert_eq!(response_dto.responded_at, invitation.responded_at.unwrap());
    }

    #[test]
    fn test_invitation_update_response_dto_from_invitation_without_response() {
        let sent_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let invitation = Invitation {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Accepted,
            sent_at: DateTime::from_naive_utc_and_offset(sent_dt, Utc),
            responded_at: None, // No response time set
            role_at_join: MemberRole::Admin,
        };

        let response_dto: InvitationUpdateResponseDto = invitation.clone().into();
        assert_eq!(response_dto.id, invitation.id);
        assert_eq!(response_dto.status, invitation.status);
        // Should use current time when responded_at is None
        assert!(response_dto.responded_at > invitation.sent_at);
    }

    #[test]
    fn test_invitation_status_enum_values() {
        assert_eq!(InvitationStatus::Pending.to_string(), "pending");
        assert_eq!(InvitationStatus::Accepted.to_string(), "accepted");
        assert_eq!(InvitationStatus::Rejected.to_string(), "rejected");
    }

    #[test]
    fn test_invitation_status_default() {
        let default_status = InvitationStatus::default();
        assert_eq!(default_status, InvitationStatus::Pending);
    }

    #[test]
    fn test_invitation_status_serialization() {
        let status = InvitationStatus::Accepted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"accepted\"");

        let deserialized: InvitationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_all_dto_equality() {
        let create_dto1 = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };
        let create_dto2 = InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };
        assert_eq!(create_dto1, create_dto2);

        let update_dto1 = InvitationUpdateStatusDto {
            status: InvitationStatus::Accepted,
            invitation_id: 1,
        };
        let update_dto2 = InvitationUpdateStatusDto {
            status: InvitationStatus::Accepted,
            invitation_id: 1,
        };
        assert_eq!(update_dto1, update_dto2);
    }

    #[test]
    fn test_set_group_membership_id() {
        let mut response_dto = InvitationUpdateResponseDto {
            id: 1,
            status: InvitationStatus::Accepted,
            responded_at: Utc::now(),
            group_membership_id: None,
        };

        assert!(response_dto.group_membership_id.is_none());
        response_dto.set_group_membership_id(42);
        assert_eq!(response_dto.group_membership_id, Some(42));
    }
}