use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemberRole {
    Member,
    Admin,
}

impl MemberRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            MemberRole::Member => "Membro",
            MemberRole::Admin => "Amministratore",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InviteMemberRequest {
    pub username: String,
    pub role: MemberRole,
}

#[component]
pub fn InviteMemberModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_invite: Callback<InviteMemberRequest>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
    #[prop(into, optional)] group_name: Option<String>,
) -> impl IntoView {
    view! { <div>"Test Invite Modal"</div> }
}
