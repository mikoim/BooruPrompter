use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn StatusBar() -> Element {
    let state: AppState = use_context();
    let status = state.status.read().clone();
    let progress = *state.progress.read();

    rsx! {
        footer { class: "status-bar",
            div { class: "status-text", { if status.is_empty() { "\u{00a0}".to_string() } else { status.clone() } } }
            div { class: "progress-shell",
                div { class: "progress-track" }
                div { class: "progress-bar", style: format_args!("width: {}%;", progress) }
                span { class: "progress-label", "{progress}%" }
            }
        }
    }
}
