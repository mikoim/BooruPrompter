use dioxus::prelude::*;

use crate::models::SortMode;
use crate::state::AppState;

#[component]
pub fn Toolbar() -> Element {
    let state: AppState = use_context();
    let active_sort = *state.sort_mode.read();

    let sort_modes = [
        SortMode::Original,
        SortMode::Custom,
        SortMode::Alphabetical,
        SortMode::Favorite,
        SortMode::Category,
    ];

    let mut clear_state = state;
    let copy_state = state;
    let paste_state = state;

    rsx! {
        nav { class: "toolbar",
            div { class: "toolbar-group",
                button { class: "ghost", onclick: move |_| clear_state.clear_prompt(), "クリア" }
                button { class: "ghost", onclick: move |_| copy_state.copy_prompt(), "コピー" }
                button { class: "ghost", onclick: move |_| paste_state.paste_prompt(), "貼り付け" }
            }
            div { class: "toolbar-group sorts",
                span { class: "label", "並べ替え" }
                for mode in sort_modes {
                    button {
                        class: format_args!("pill {}", if active_sort == mode { "active" } else { "" }),
                        onclick: {
                            let mut state = state;
                            move |_| state.set_sort_mode(mode)
                        },
                        "{mode.label()}"
                    }
                }
            }
        }
    }
}
