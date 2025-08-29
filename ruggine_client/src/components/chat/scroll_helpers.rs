use leptos::*;
use leptos::html::Div;
use web_sys::Element;

// Pixel offset constants used by ChatView scroll logic
pub const ANCHOR_OFFSET_PX: f64 = 24.0;
pub const ANCHOR_EXTRA_OFFSET_PX: f64 = 48.0;

// Helper: check whether an element is visible inside a scrolling container
pub fn is_element_in_viewport(container: &leptos::HtmlElement<Div>, element: &Element) -> bool {
    let container_rect = container.get_bounding_client_rect();
    let elem_rect = element.get_bounding_client_rect();
    elem_rect.top() < container_rect.bottom() && elem_rect.bottom() > container_rect.top()
}

// Compute desired scrollTop to align element bottom near container bottom (bottom-aligned anchor)
pub fn compute_bottom_aligned_scroll(container: &leptos::HtmlElement<Div>, element: &Element) -> i32 {
    let elem_rect = element.get_bounding_client_rect();
    let container_rect = container.get_bounding_client_rect();
    let current_top = container.scroll_top() as f64;
    let desired = current_top + (elem_rect.bottom() - container_rect.bottom()) + ANCHOR_OFFSET_PX - ANCHOR_EXTRA_OFFSET_PX;
    desired as i32
}

// Decrement unread counter helper for a given group id
pub fn decrement_unread_for_group(unread_counts: &leptos::RwSignal<std::collections::HashMap<i32, u32>>, group_id: i32) {
    unread_counts.update(|map| {
        let entry = map.entry(group_id).or_insert(0);
        if *entry > 0 {
            *entry -= 1;
        }
    });
}
