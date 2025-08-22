use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::entity::group_membership::{CurrentAction, MemberRole, MembershipStatus};

#[derive(Debug, sqlx::FromRow, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupMembershipWithInvitationRow {
    pub id: i32,
    pub user_id: i32,           // from `invitation.to_user_id`
    pub group_chat_id: i32,     // from `invitation.group_chat_id`
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub membership_status: MembershipStatus,
    pub invitation_id: i32,
    pub current_action: CurrentAction,
}
