use leptos::*;
use crate::hooks::use_groups::{UseGroups, use_groups};

/// Context for providing groups hook across components
#[derive(Clone)]
pub struct GroupsContext {
    pub groups_hook: UseGroups,
}

/// Provide groups context
pub fn provide_groups_context() -> GroupsContext {
    let groups_hook = use_groups();
    let context = GroupsContext {
        groups_hook: groups_hook.clone(),
    };
    
    provide_context(context.clone());
    context
}

/// Use groups context from provider
pub fn use_groups_context() -> GroupsContext {
    expect_context::<GroupsContext>()
}
