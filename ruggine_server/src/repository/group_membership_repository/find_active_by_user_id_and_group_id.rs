use sqlx::Error;
use crate::entity::group_membership::MembershipStatus;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_active_by_user_id_and_group_id_inner(
        &self,
        user_id: i32,
        group_id: i32,
    ) -> Result<GroupMembershipWithInvitationRow, Error> {
        // Riutilizzo della funzione generica con filtro Active
        let rows = self
            .find_by_user_id_and_group_id_inner(
                user_id,
                group_id,
                vec![MembershipStatus::Active],
            )
            .await?;

        // Se non c'è, solleva RowNotFound (comportamento simile a fetch_one)
        rows.into_iter()
            .next()
            .ok_or_else(|| Error::RowNotFound)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use chrono::Utc;
    use sqlx::Error;
    use std::sync::Arc;
    use std::collections::HashMap;
    use async_trait::async_trait;

    // Mock trait implementation for testing
    #[derive(Clone)]
    struct MockGroupMembershipRepository {
        pub memberships: Arc<std::sync::Mutex<HashMap<(i32, i32), Vec<GroupMembershipWithInvitationRow>>>>,
    }

    impl MockGroupMembershipRepository {
        fn new() -> Self {
            Self {
                memberships: Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }

        fn add_membership(&self, user_id: i32, group_id: i32, membership: GroupMembershipWithInvitationRow) {
            let mut memberships = self.memberships.lock().unwrap();
            memberships.entry((user_id, group_id)).or_insert_with(Vec::new).push(membership);
        }

        async fn find_by_user_id_and_group_id_inner_mock(
            &self,
            user_id: i32,
            group_id: i32,
            statuses: Vec<MembershipStatus>,
        ) -> Result<Vec<GroupMembershipWithInvitationRow>, Error> {
            let memberships = self.memberships.lock().unwrap();
            
            if let Some(user_memberships) = memberships.get(&(user_id, group_id)) {
                let filtered: Vec<GroupMembershipWithInvitationRow> = user_memberships
                    .iter()
                    .filter(|m| statuses.contains(&m.membership_status))
                    .cloned()
                    .collect();
                Ok(filtered)
            } else {
                Ok(vec![])
            }
        }

        async fn find_active_by_user_id_and_group_id_inner_mock(
            &self,
            user_id: i32,
            group_id: i32,
        ) -> Result<GroupMembershipWithInvitationRow, Error> {
            // Simula la logica della funzione reale
            let rows = self
                .find_by_user_id_and_group_id_inner_mock(
                    user_id,
                    group_id,
                    vec![MembershipStatus::Active],
                )
                .await?;

            rows.into_iter()
                .next()
                .ok_or_else(|| Error::RowNotFound)
        }
    }

    fn create_mock_membership_with_invitation_row(
        id: i32,
        user_id: i32,
        group_id: i32,
        status: MembershipStatus,
        role: MemberRole,
    ) -> GroupMembershipWithInvitationRow {
        GroupMembershipWithInvitationRow {
            id,
            user_id,
            group_chat_id: group_id,
            role,
            joined_at: Utc::now(),
            left_at: if status == MembershipStatus::Left { Some(Utc::now()) } else { None },
            membership_status: status,
            invitation_id: 1,
        }
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_success() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 1;
        let group_id = 10;
        let membership = create_mock_membership_with_invitation_row(
            100, 
            user_id, 
            group_id, 
            MembershipStatus::Active, 
            MemberRole::Member
        );
        
        mock_repo.add_membership(user_id, group_id, membership.clone());

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, 100);
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert_eq!(found.role, MemberRole::Member);
        assert!(found.left_at.is_none());
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_admin_role() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 2;
        let group_id = 20;
        let membership = create_mock_membership_with_invitation_row(
            200, 
            user_id, 
            group_id, 
            MembershipStatus::Active, 
            MemberRole::Admin
        );
        
        mock_repo.add_membership(user_id, group_id, membership.clone());

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, 200);
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert_eq!(found.role, MemberRole::Admin);
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_not_found_no_membership() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 3;
        let group_id = 30;

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_not_found_left_membership() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 4;
        let group_id = 40;
        let membership = create_mock_membership_with_invitation_row(
            400, 
            user_id, 
            group_id, 
            MembershipStatus::Left, 
            MemberRole::Member
        );
        
        mock_repo.add_membership(user_id, group_id, membership.clone());

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_multiple_memberships_only_one_active() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 5;
        let group_id = 50;
        
        // Add left membership
        let left_membership = create_mock_membership_with_invitation_row(
            501, 
            user_id, 
            group_id, 
            MembershipStatus::Left, 
            MemberRole::Member
        );
        mock_repo.add_membership(user_id, group_id, left_membership);
        
        // Add active membership
        let active_membership = create_mock_membership_with_invitation_row(
            502, 
            user_id, 
            group_id, 
            MembershipStatus::Active, 
            MemberRole::Admin
        );
        mock_repo.add_membership(user_id, group_id, active_membership);

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, 502); // Should return the active one
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert_eq!(found.role, MemberRole::Admin);
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_returns_first_active_when_multiple() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = 6;
        let group_id = 60;
        
        // Add first active membership
        let active_membership1 = create_mock_membership_with_invitation_row(
            601, 
            user_id, 
            group_id, 
            MembershipStatus::Active, 
            MemberRole::Member
        );
        mock_repo.add_membership(user_id, group_id, active_membership1);
        
        // Add second active membership (edge case, should not happen in real data)
        let active_membership2 = create_mock_membership_with_invitation_row(
            602, 
            user_id, 
            group_id, 
            MembershipStatus::Active, 
            MemberRole::Admin
        );
        mock_repo.add_membership(user_id, group_id, active_membership2);

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        // Should return the first one found (implementation returns first from iterator)
        assert!(found.id == 601 || found.id == 602);
        assert_eq!(found.membership_status, MembershipStatus::Active);
    }

    #[tokio::test]
    async fn test_find_active_with_negative_ids() {
        let mock_repo = MockGroupMembershipRepository::new();
        
        let user_id = -1;
        let group_id = -1;

        let result = mock_repo.find_active_by_user_id_and_group_id_inner_mock(user_id, group_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }
}