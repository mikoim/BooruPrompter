use dioxus::prelude::*;

mod components;
mod dictionary;
mod models;
mod sort;
mod state;
mod tags;
mod utils;

use crate::state::{hydrate_from_storage, load_dictionary_async, use_app_state};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const PAGE_TITLE: &str = "BooruPrompter Online";
const PAGE_DESCRIPTION: &str =
    "Three-pane prompt editor with booru-style tag suggestions, sorting, and clipboard helpers.";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let state = use_app_state();
    use_context_provider(|| state.clone());

    use_effect({
        let state = state.clone();
        move || {
            spawn(load_dictionary_async(state.clone()));
            spawn(hydrate_from_storage(state.clone()));
        }
    });

    rsx! {
        document::Title { "{PAGE_TITLE}" }
        document::Meta { charset: "UTF-8" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Meta { name: "description", content: PAGE_DESCRIPTION }
        document::Meta { name: "robots", content: "index, follow" }
        document::Meta { property: "og:title", content: PAGE_TITLE }
        document::Meta { property: "og:description", content: PAGE_DESCRIPTION }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: PAGE_TITLE }
        document::Meta { name: "twitter:description", content: PAGE_DESCRIPTION }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        components::AppShell {}
    }
}
