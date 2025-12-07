use dioxus::prelude::*;

use crate::models::TagMove;
use crate::state::AppState;
use crate::utils::category_class;

#[component]
pub fn TagList() -> Element {
    let state: AppState = use_context();
    let tags = state.tags.read().clone();
    let count = tags.iter().filter(|t| t.text != "\n").count();

    let list_body = if tags.is_empty() {
        rsx! { p { class: "muted", "タグはまだありません" } }
    } else {
        rsx! {
            for (idx, tag) in tags.iter().enumerate() {
                div { key: "{idx}-{tag.text}", class: if tag.text == "\n" { "row break" } else { "row" },
                    div { class: format_args!("tag {}", category_class(tag.category)), { if tag.text == "\n" { "(改行)".to_string() } else { tag.text.clone() } } }
                    div { class: "desc", "{tag.description.clone().unwrap_or_default()}" }
                    div { class: "actions",
                        if tag.text != "\n" {
                            button { class: "icon", onclick: { let mut state = state; move |_| state.move_tag(idx, TagMove::Top) }, "↟" }
                            button { class: "icon", onclick: { let mut state = state; move |_| state.move_tag(idx, TagMove::Up) }, "↑" }
                            button { class: "icon", onclick: { let mut state = state; move |_| state.move_tag(idx, TagMove::Down) }, "↓" }
                            button { class: "icon", onclick: { let mut state = state; move |_| state.move_tag(idx, TagMove::Bottom) }, "↡" }
                            button { class: "icon danger", onclick: { let mut state = state; move |_| state.delete_tag(idx) }, "✕" }
                        } else {
                            span { class: "muted", "固定" }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "pane-header",
            div { class: "title",
                h3 { "Tag List" }
                span { class: "hint", "{count} items" }
            }
        }
        div { class: "list tag-list", {list_body} }
    }
}
