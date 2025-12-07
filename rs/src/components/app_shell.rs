use dioxus::prelude::*;

use crate::models::LayoutMode;

use crate::components::PromptEditor;
use crate::components::StatusBar;
use crate::components::SuggestionList;
use crate::components::TagList;
use crate::components::Toolbar;

#[component]
pub fn AppShell() -> Element {
    let layout = LayoutMode::Stacked;

    rsx! {
        div { class: "app-shell",
            header { class: "hero",
                div { class: "branding",
                    span { class: "eyebrow", "BooruPrompter" }
                    h1 { "Prompt Lab" }
                    p { "3-pane prompt editor with built-in booru suggestions" }
                }
            }
            Toolbar {}
            main { class: format_args!("workspace {}", layout.class()),
                section { class: "panel prompt-panel", PromptEditor {} }
                section { class: "panel suggestion-panel", SuggestionList {} }
                section { class: "panel tags-panel", TagList {} }
            }
            StatusBar {}
        }
    }
}
