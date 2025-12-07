use std::sync::Arc;

use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::TimeoutFuture;

use crate::dictionary::{generate_suggestions, load_dictionary};
use crate::models::{Dictionary, DictionaryState, SortMode, TagEntry, TagMove, TagSuggestion};
use crate::sort::{sort_from_string, sort_tags, sort_to_string};
use crate::tags::{decorate_tags, extract_tags, rebuild_prompt_from_tags};
use crate::utils::{
    byte_to_utf16_index, copy_to_clipboard, editor_element, needs_trailing_comma,
    read_from_clipboard, requires_leading_space, token_at_cursor, token_bounds, trim_bounds,
    utf16_to_byte_index,
};

pub const STORAGE_PROMPT_KEY: &str = "booru_prompter.prompt";
pub const STORAGE_SORT_KEY: &str = "booru_prompter.sort";

#[derive(Clone, Copy)]
pub struct AppState {
    pub prompt: Signal<String>,
    pub tags: Signal<Vec<TagEntry>>,
    pub suggestions: Signal<Vec<TagSuggestion>>,
    pub status: Signal<String>,
    pub progress: Signal<u8>,
    pub dictionary: Signal<DictionaryState>,
    pub sort_mode: Signal<SortMode>,
    pub cursor_utf16: Signal<usize>,
    pub request_nonce: Signal<u64>,
    pub last_query: Signal<String>,
}

pub fn use_app_state() -> AppState {
    AppState {
        prompt: use_signal(String::new),
        tags: use_signal(Vec::new),
        suggestions: use_signal(Vec::new),
        status: use_signal(|| "辞書を読み込み中...".to_string()),
        progress: use_signal(|| 0),
        dictionary: use_signal(|| DictionaryState::Loading),
        sort_mode: use_signal(|| SortMode::Original),
        cursor_utf16: use_signal(|| 0),
        request_nonce: use_signal(|| 0),
        last_query: use_signal(String::new),
    }
}

impl AppState {
    pub fn clear_prompt(&mut self) {
        self.prompt.set(String::new());
        self.tags.set(Vec::new());
        self.suggestions.set(Vec::new());
        self.status.set(String::new());
        self.progress.set(0);
        self.persist_prompt();
    }

    pub fn copy_prompt(&self) {
        let text = self.prompt.read().clone();
        let mut state = *self;
        spawn(async move {
            match copy_to_clipboard(&text).await {
                Ok(_) => {
                    state.status.set("コピーしました".to_string());
                    state.progress.set(100);
                }
                Err(err) => {
                    state.status.set(format!("コピーできません: {err}"));
                }
            }
        });
    }

    pub fn paste_prompt(&self) {
        let mut state = *self;
        spawn(async move {
            match read_from_clipboard().await {
                Ok(text) => {
                    state.prompt.set(text.clone());
                    state.persist_prompt();
                    state.sync_tags();
                    state.suggestions.set(Vec::new());
                    state.status.set("貼り付けました".to_string());
                }
                Err(err) => {
                    state.status.set(format!("貼り付けに失敗: {err}"));
                }
            }
        });
    }

    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode.set(mode);
        self.persist_sort_mode();
        self.apply_sort(mode);
    }

    pub fn apply_sort(&mut self, mode: SortMode) {
        let dict = self.dictionary_ready();
        let current = self.tags.read().clone();
        if current.is_empty() {
            return;
        }
        let sorted = sort_tags(&current, mode, dict.as_deref());
        self.apply_tag_edits(sorted);
    }

    pub fn move_tag(&mut self, index: usize, target: TagMove) {
        let mut tags = self.tags.read().clone();
        if index >= tags.len() {
            return;
        }
        if tags[index].text == "\n" {
            return;
        }
        let new_index = match target {
            TagMove::Up => index.saturating_sub(1),
            TagMove::Down => (index + 1).min(tags.len().saturating_sub(1)),
            TagMove::Top => 0,
            TagMove::Bottom => tags.len().saturating_sub(1),
        };
        tags.swap(index, new_index);
        self.apply_tag_edits(tags);
    }

    pub fn delete_tag(&mut self, index: usize) {
        let mut tags = self.tags.read().clone();
        if index >= tags.len() || tags[index].text == "\n" {
            return;
        }
        tags.remove(index);
        self.apply_tag_edits(tags);
    }

    pub fn on_prompt_input(&mut self, text: String) {
        self.prompt.set(text.clone());
        self.persist_prompt();
        self.clear_progress();
        self.sync_tags();

        let cursor_utf16 = *self.cursor_utf16.peek();
        let cursor_byte = utf16_to_byte_index(&text, cursor_utf16);
        let token = token_at_cursor(&text, cursor_byte);

        if token.is_empty() {
            self.status.set(String::new());
            self.suggestions.set(Vec::new());
            return;
        }

        self.status.set(format!("サジェスト中: {}", token.trim()));
        if token == *self.last_query.peek() {
            return;
        }
        self.last_query.set(token.clone());
        self.schedule_suggestion(token);
    }

    pub fn capture_cursor(&mut self) {
        #[cfg(feature = "web")]
        if let Some(element) = editor_element() {
            if let Ok(Some(pos)) = element.selection_start() {
                self.cursor_utf16.set(pos as usize);
                return;
            }
        }
        self.cursor_utf16
            .set(self.prompt.read().encode_utf16().count());
    }

    pub fn insert_suggestion(&mut self, tag: &str) {
        let prompt = self.prompt.read().clone();
        let cursor_utf16 = *self.cursor_utf16.peek();
        let cursor_byte = utf16_to_byte_index(&prompt, cursor_utf16);

        let (start, end) = token_bounds(&prompt, cursor_byte);
        let (trimmed_start, trimmed_end) = trim_bounds(&prompt, start, end);

        let before = &prompt[..trimmed_start];
        let after = &prompt[trimmed_end..];

        let needs_space = requires_leading_space(before);
        let needs_trailing = needs_trailing_comma(after);

        let mut replacement = String::new();
        if needs_space {
            replacement.push(' ');
        }
        replacement.push_str(tag);
        if needs_trailing {
            replacement.push_str(", ");
        }

        let mut next_prompt = String::new();
        next_prompt.push_str(&prompt[..trimmed_start]);
        next_prompt.push_str(&replacement);
        next_prompt.push_str(after);

        self.prompt.set(next_prompt.clone());
        self.persist_prompt();
        self.suggestions.set(Vec::new());
        self.sync_tags();

        let caret_byte =
            trimmed_start + replacement.trim_end_matches(|c| c == ' ' || c == ',').len();
        #[cfg(feature = "web")]
        if let Some(element) = editor_element() {
            let caret_utf16 = byte_to_utf16_index(&next_prompt, caret_byte);
            let _ = element.set_selection_range(caret_utf16 as u32, caret_utf16 as u32);
            let _ = element.focus();
            self.cursor_utf16.set(caret_utf16);
        }
    }

    pub fn sync_tags(&mut self) {
        let text = self.prompt.read().clone();
        let mut tags = extract_tags(&text);
        if let Some(dict) = self.dictionary_ready() {
            decorate_tags(&mut tags, dict.as_ref());
        }
        self.tags.set(tags);
    }

    fn apply_tag_edits(&mut self, tags: Vec<TagEntry>) {
        let prompt = rebuild_prompt_from_tags(&tags);
        self.prompt.set(prompt.clone());
        self.persist_prompt();
        self.sync_tags();
        self.suggestions.set(Vec::new());
    }

    fn schedule_suggestion(&mut self, token: String) {
        let request_id = {
            let mut nonce = self.request_nonce.write();
            *nonce += 1;
            *nonce
        };
        let mut state = *self;
        spawn(async move {
            TimeoutFuture::new(30).await;
            if *state.request_nonce.peek() != request_id {
                return;
            }
            let Some(dict) = state.dictionary_ready() else {
                return;
            };
            let suggestions = generate_suggestions(token.trim(), dict.as_ref());
            if suggestions.is_empty() {
                state.status.set("サジェストなし".to_string());
            }
            state.suggestions.set(suggestions);
        });
    }

    fn dictionary_ready(&self) -> Option<Arc<Dictionary>> {
        match &*self.dictionary.read() {
            DictionaryState::Ready(dict) => Some(dict.clone()),
            _ => None,
        }
    }

    fn persist_prompt(&mut self) {
        #[cfg(feature = "web")]
        if let Err(err) = LocalStorage::set(STORAGE_PROMPT_KEY, self.prompt.read().clone()) {
            self.status
                .set(format!("localStorage に保存できません: {err:?}"));
        }
    }

    fn persist_sort_mode(&self) {
        #[cfg(feature = "web")]
        let _ = LocalStorage::set(STORAGE_SORT_KEY, sort_to_string(*self.sort_mode.peek()));
    }

    fn clear_progress(&mut self) {
        self.progress.set(0);
        self.status.set(String::new());
    }
}

pub async fn hydrate_from_storage(mut state: AppState) {
    #[cfg(feature = "web")]
    {
        if let Ok(saved_prompt) = LocalStorage::get::<String>(STORAGE_PROMPT_KEY) {
            state.prompt.set(saved_prompt.clone());
            state.sync_tags();
        }
        if let Ok(saved_sort) = LocalStorage::get::<String>(STORAGE_SORT_KEY) {
            state.sort_mode.set(sort_from_string(&saved_sort));
        }
    }
}

pub async fn load_dictionary_async(mut state: AppState) {
    state.status.set("辞書を読み込み中...".to_string());
    match load_dictionary() {
        Ok(dict) => {
            state.dictionary.set(DictionaryState::Ready(Arc::new(dict)));
            state.status.set("辞書を読み込みました".to_string());
            state.sync_tags();
        }
        Err(err) => {
            state.dictionary.set(DictionaryState::Failed(err.clone()));
            state
                .status
                .set(format!("辞書の読み込みに失敗しました: {err}"));
        }
    }
}
