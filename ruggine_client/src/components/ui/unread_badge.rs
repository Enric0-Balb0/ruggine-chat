use leptos::*;

#[component]
pub fn UnreadBadge(
    #[prop(into)] count: i32,
) -> impl IntoView {
    let style = if count > 0 {
        ""
    } else {
        "display: none;"
    };
    view! {
        <span style=style class="ml-2 px-2 py-0.5 text-xs font-bold bg-red-600 text-white rounded-full min-w-[1.5em] text-center">
            {count}
        </span>
    }
}
