use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::Utc;

use crate::{
    dto::invitation_dto::{InvitationCreateDto, InvitationReadDto, InvitationUpdateStatusDto, InvitationUpdateResponseDto},
    entity::invitation::{NewInvitation, Invitation, UpdateInvitationStatus, InvitationStatus}
};
use crate::entity::group_membership::MemberRole;

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct InvitationFactory;

impl InvitationFactory {

    pub fn fake_new_invitation() -> NewInvitation {
        NewInvitation {
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_new_admin_invitation() -> NewInvitation {
        NewInvitation {
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Admin,
        }
    }

    pub fn fake_invitation() -> Invitation {
        Invitation {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: Utc::now(),
            responded_at: None,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_admin_invitation() -> Invitation {
        Invitation {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: Utc::now(),
            responded_at: None,
            role_at_join: MemberRole::Admin,
        }
    }

    pub fn fake_invitation_with_status(status: InvitationStatus) -> Invitation {
        let mut invitation = Self::fake_invitation();
        invitation.status = status;
        if invitation.status != InvitationStatus::Pending {
            invitation.responded_at = Some(Utc::now());
        }
        invitation
    }

    pub fn fake_invitation_from_ids(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        Invitation {
            id: counter as i32,
            from_user_id,
            to_user_id,
            group_chat_id,
            status: InvitationStatus::Pending,
            sent_at: Utc::now(),
            responded_at: None,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_admin_invitation_from_ids(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        Invitation {
            id: counter as i32,
            from_user_id,
            to_user_id,
            group_chat_id,
            status: InvitationStatus::Pending,
            sent_at: Utc::now(),
            responded_at: None,
            role_at_join: MemberRole::Admin,
        }
    }

    pub fn fake_invitation_create_dto() -> InvitationCreateDto {
        InvitationCreateDto {
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_admin_invitation_create_dto() -> InvitationCreateDto {
        let mut invitation = Self::fake_invitation_create_dto();
        invitation.role_at_join = MemberRole::Admin;
        invitation
    }

    pub fn fake_invitation_create_dto_with_ids(to_user_id: i32, group_chat_id: i32) -> InvitationCreateDto {
        InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_admin_invitation_create_dto_with_ids(to_user_id: i32, group_chat_id: i32) -> InvitationCreateDto {
        let mut invitation_create_dto = Self::fake_invitation_create_dto_with_ids(to_user_id, group_chat_id);
        invitation_create_dto.role_at_join = MemberRole::Admin;
        invitation_create_dto
    }

    pub fn fake_invitation_read_dto() -> InvitationReadDto {
        use chrono::{NaiveDate, DateTime, Utc};

        let naive_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        InvitationReadDto {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Pending,
            sent_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            responded_at: None,
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_invitation_admin_read_dto() -> InvitationReadDto {
        let mut invitation = Self::fake_invitation_read_dto();
        invitation.role_at_join = MemberRole::Admin;
        invitation
    }

    pub fn fake_invitation_read_dto_with_response() -> InvitationReadDto {
        use chrono::{NaiveDate, DateTime, Utc};

        let sent_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let responded_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        InvitationReadDto {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            status: InvitationStatus::Accepted,
            sent_at: DateTime::from_naive_utc_and_offset(sent_dt, Utc),
            responded_at: Some(DateTime::from_naive_utc_and_offset(responded_dt, Utc)),
            role_at_join: MemberRole::Member,
        }
    }

    pub fn fake_admin_invitation_read_dto_with_response() -> InvitationReadDto {
        let mut invitation = Self::fake_invitation_read_dto_with_response();
        invitation.role_at_join = MemberRole::Admin;
        invitation
    }

    pub fn fake_invitation_update_status_dto() -> InvitationUpdateStatusDto {
        InvitationUpdateStatusDto {
            status: InvitationStatus::Accepted,
            invitation_id: 1
        }
    }

    pub fn fake_invitation_update_status_dto_rejected() -> InvitationUpdateStatusDto {
        InvitationUpdateStatusDto {
            status: InvitationStatus::Rejected,
            invitation_id: 1
        }
    }

    pub fn fake_invitation_update_response_dto() -> InvitationUpdateResponseDto {
        use chrono::{NaiveDate, DateTime, Utc};

        let naive_dt = NaiveDate::from_ymd_opt(2025, 1, 28)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        InvitationUpdateResponseDto {
            id: 1,
            status: InvitationStatus::Accepted,
            responded_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            group_membership_id: Some(1),
        }
    }

    pub fn fake_update_invitation_status() -> UpdateInvitationStatus {
        UpdateInvitationStatus {
            status: InvitationStatus::Accepted,
            invitation_id: 1
        }
    }

    pub fn fake_update_invitation_status_rejected() -> UpdateInvitationStatus {
        UpdateInvitationStatus {
            status: InvitationStatus::Rejected,
            invitation_id: 1
        }
    }

    // Utility methods for testing
    pub fn with_from_user_id(mut invitation: NewInvitation, from_user_id: i32) -> NewInvitation {
        invitation.from_user_id = from_user_id;
        invitation
    }

    pub fn with_to_user_id(mut invitation: NewInvitation, to_user_id: i32) -> NewInvitation {
        invitation.to_user_id = to_user_id;
        invitation
    }

    pub fn with_group_chat_id(mut invitation: NewInvitation, group_chat_id: i32) -> NewInvitation {
        invitation.group_chat_id = group_chat_id;
        invitation
    }

    pub fn with_status(mut invitation: Invitation, status: InvitationStatus) -> Invitation {
        invitation.status = status;
        if invitation.status != InvitationStatus::Pending {
            invitation.responded_at = Some(Utc::now());
        } else {
            invitation.responded_at = None;
        }
        invitation
    }

    pub fn with_id(mut invitation: Invitation, id: i32) -> Invitation {
        invitation.id = id;
        invitation
    }

    // Utility methods for DTOs
    pub fn with_to_user_id_dto(mut dto: InvitationCreateDto, to_user_id: i32) -> InvitationCreateDto {
        dto.to_user_id = to_user_id;
        dto
    }

    pub fn with_group_chat_id_dto(mut dto: InvitationCreateDto, group_chat_id: i32) -> InvitationCreateDto {
        dto.group_chat_id = group_chat_id;
        dto
    }

    pub fn with_status_dto(mut dto: InvitationUpdateStatusDto, status: InvitationStatus) -> InvitationUpdateStatusDto {
        dto.status = status;
        dto
    }

    // Helper methods for creating specific scenarios
    pub fn create_accepted_invitation(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
        let mut invitation = Self::fake_invitation();
        invitation.from_user_id = from_user_id;
        invitation.to_user_id = to_user_id;
        invitation.group_chat_id = group_chat_id;
        invitation.status = InvitationStatus::Accepted;
        invitation.responded_at = Some(Utc::now());
        invitation
    }

    pub fn create_rejected_invitation(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
        let mut invitation = Self::fake_invitation();
        invitation.from_user_id = from_user_id;
        invitation.to_user_id = to_user_id;
        invitation.group_chat_id = group_chat_id;
        invitation.status = InvitationStatus::Rejected;
        invitation.responded_at = Some(Utc::now());
        invitation
    }

    pub fn create_pending_invitation(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
        let mut invitation = Self::fake_invitation();
        invitation.from_user_id = from_user_id;
        invitation.to_user_id = to_user_id;
        invitation.group_chat_id = group_chat_id;
        invitation.status = InvitationStatus::Pending;
        invitation.responded_at = None;
        invitation
    }
}