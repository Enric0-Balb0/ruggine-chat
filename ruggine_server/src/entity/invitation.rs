use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq, Eq)]
pub struct Invitation {
    pub id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub group_chat_id: i32,
    pub status: InvitationStatus,
    pub sent_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewInvitation {
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub group_chat_id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateInvitationStatus {
    pub status: InvitationStatus,
    pub responded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "invitation_status")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Rejected,
}

impl Default for InvitationStatus {
    fn default() -> Self {
        InvitationStatus::Pending
    }
}

impl Invitation {
    /// Creates a new invitation with pending status
    pub fn create(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> NewInvitation {
        NewInvitation {
            from_user_id,
            to_user_id,
            group_chat_id,
        }
    }

    /// Updates the invitation status and sets responded_at timestamp
    pub fn update_status(status: InvitationStatus) -> UpdateInvitationStatus {
        UpdateInvitationStatus {
            status,
            responded_at: Utc::now(),
        }
    }

    /// Checks if the invitation is pending
    pub fn is_pending(&self) -> bool {
        self.status == InvitationStatus::Pending
    }

    /// Checks if the invitation is accepted
    pub fn is_accepted(&self) -> bool {
        self.status == InvitationStatus::Accepted
    }

    /// Checks if the invitation is rejected
    pub fn is_rejected(&self) -> bool {
        self.status == InvitationStatus::Rejected
    }

    /// Checks if the invitation has been responded to
    pub fn is_responded(&self) -> bool {
        self.responded_at.is_some()
    }
}
