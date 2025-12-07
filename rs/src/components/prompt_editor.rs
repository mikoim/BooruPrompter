use dioxus::prelude::*;

use crate::state::AppState;
use crate::utils::EDITOR_ID;

#[component]
pub fn PromptEditor() -> Element {
    let state: AppState = use_context();
    let prompt_text = state.prompt.read().clone();
    let tag_count = state.tags.read().iter().filter(|t| t.text != "\n").count();
    let mut state_for_input = state;
    let mut state_for_click = state;
    let mut state_for_key = state;
    let mut state_for_mouse = state;

    rsx! {
        div { class: "pane-header",
            div { class: "title",
                h3 { "Prompt Editor" }
                span { class: "hint", "{tag_count} tags" }
            }
            span { class: "hint subtle", "入力は 30ms デバウンス" }
        }
        textarea {
            id: EDITOR_ID,
            class: "prompt-area",
            value: prompt_text,
            placeholder: "タグをカンマ区切りで入力...",
            oninput: move |evt| {
                state_for_input.capture_cursor();
                state_for_input.on_prompt_input(evt.value());
            },
            onclick: move |_| state_for_click.capture_cursor(),
            onkeyup: move |_| state_for_key.capture_cursor(),
            onmouseup: move |_| state_for_mouse.capture_cursor(),
        }
    }
}
