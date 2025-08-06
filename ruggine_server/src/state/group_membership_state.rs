use std::sync::Arc;
use crate::config::database::Database;
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};

#[derive(Clone)]
pub struct GroupMembershipState {
    pub(crate) group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
}

impl GroupMembershipState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let group_membership_service = Arc::new(GroupMembershipService::new(db_conn));

        Self {
            group_membership_service
        }
    }

    pub fn with_dependencies(
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            group_membership_service,
        }
    }
}
