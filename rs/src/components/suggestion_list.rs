use dioxus::prelude::*;

use crate::models::DictionaryState;
use crate::state::AppState;
use crate::utils::category_class;

#[component]
pub fn SuggestionList() -> Element {
    let state: AppState = use_context();
    let suggestions = state.suggestions.read().clone();
    let dict_state = state.dictionary.read().clone();

    let content = match dict_state {
        DictionaryState::Loading => rsx! { p { class: "muted", "辞書を読み込み中..." } },
        DictionaryState::Failed(msg) => rsx! { p { class: "muted", "辞書エラー: {msg}" } },
        DictionaryState::Ready(_) => {
            if suggestions.is_empty() {
                rsx! { p { class: "muted", "入力中のトークンに応じてサジェストします" } }
            } else {
                rsx! {
                    for (idx, item) in suggestions.iter().enumerate() {
                        div {
                            key: "{idx}-{item.tag}",
                            class: "row clickable",
                            onclick: {
                                let tag = item.tag.clone();
                                let mut state = state;
                                move |_| state.insert_suggestion(&tag)
                            },
                            div { class: format_args!("tag {}", category_class(item.category)), "{item.tag}" }
                            div { class: "desc", "{item.description.clone().unwrap_or_default()}" }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "pane-header",
            div { class: "title",
                h3 { "Suggestion" }
                span { class: "hint", "クリックまたは Enter で挿入" }
            }
        }
        div { class: "list suggestion-list", {content} }
    }
}
