use leptos::*;
use crate::types::group::GroupChatCreateRequest;

#[component]
pub fn CreateGroupModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_create: Callback<GroupChatCreateRequest>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
) -> impl IntoView {
    view! { <div>"Test Create Group Modal"</div> }
}
