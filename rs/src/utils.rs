use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "web")]
use web_sys::HtmlTextAreaElement;
use web_sys::{wasm_bindgen::JsCast, window};

pub const EDITOR_ID: &str = "prompt-editor";

/// Formats a description with a category suffix when applicable.
pub fn format_description_with_suffix(desc: &str, category: i32) -> String {
    match category_suffix(category) {
        Some(suffix) if !suffix.is_empty() => format!("{desc} ({suffix})"),
        _ => desc.to_string(),
    }
}

pub fn utf8_has_multibyte(input: &str) -> bool {
    input.as_bytes().iter().any(|b| *b >= 0x80)
}

pub fn category_suffix(category: i32) -> Option<&'static str> {
    match category {
        1 => Some("Artist"),
        3 => Some("Copyright"),
        4 => Some("Character"),
        5 => Some("Metadata"),
        _ => None,
    }
}

pub fn category_class(category: i32) -> &'static str {
    match category {
        1 => "cat-artist",
        3 => "cat-copyright",
        4 => "cat-character",
        5 => "cat-metadata",
        _ => "cat-general",
    }
}

pub fn token_at_cursor(text: &str, cursor: usize) -> String {
    let (start, end) = token_bounds(text, cursor);
    text[start..end].trim().to_string()
}

pub fn token_bounds(text: &str, cursor: usize) -> (usize, usize) {
    let mut start = 0usize;
    for (idx, ch) in text.char_indices() {
        if idx >= cursor {
            break;
        }
        if ch == ',' || ch == '\n' {
            start = idx + ch.len_utf8();
        }
    }

    let mut end = text.len();
    for (idx, ch) in text.char_indices() {
        if idx < cursor {
            continue;
        }
        if ch == ',' || ch == '\n' {
            end = idx;
            break;
        }
    }
    (start, end)
}

pub fn trim_bounds(text: &str, start: usize, end: usize) -> (usize, usize) {
    let slice = &text[start..end];
    let trimmed_start = slice.len() - slice.trim_start().len();
    let trimmed_end = slice.len() - slice.trim_end().len();
    (start + trimmed_start, end - trimmed_end)
}

pub fn requires_leading_space(before: &str) -> bool {
    before
        .chars()
        .rev()
        .find(|c| !c.is_whitespace())
        .map(|c| c != ',' && c != '\n')
        .unwrap_or(false)
}

pub fn needs_trailing_comma(after: &str) -> bool {
    after
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c != ',')
        .unwrap_or(false)
}

pub fn utf16_to_byte_index(s: &str, utf16_index: usize) -> usize {
    let mut accum = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        let len16 = ch.len_utf16();
        if accum + len16 > utf16_index {
            return byte_idx;
        }
        accum += len16;
    }
    s.len()
}

pub fn byte_to_utf16_index(s: &str, byte_index: usize) -> usize {
    s[..byte_index].encode_utf16().count()
}

#[cfg(feature = "web")]
pub fn editor_element() -> Option<HtmlTextAreaElement> {
    let document = window()?.document()?;
    let element = document.get_element_by_id(EDITOR_ID)?;
    element.dyn_into::<HtmlTextAreaElement>().ok()
}

#[cfg(not(feature = "web"))]
pub fn editor_element() -> Option<HtmlTextAreaElement> {
    None
}

#[cfg(feature = "web")]
pub async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let navigator = window().ok_or("window が取得できません")?.navigator();
    let clipboard = navigator.clipboard();
    let promise = clipboard.write_text(text);
    JsFuture::from(promise)
        .await
        .map_err(|_| "clipboard へ書き込めません".to_string())?;
    Ok(())
}

#[cfg(not(feature = "web"))]
pub async fn copy_to_clipboard(_text: &str) -> Result<(), String> {
    Err("web でのみコピーできます".to_string())
}

#[cfg(feature = "web")]
pub async fn read_from_clipboard() -> Result<String, String> {
    let navigator = window().ok_or("window が取得できません")?.navigator();
    let clipboard = navigator.clipboard();
    let value = JsFuture::from(clipboard.read_text())
        .await
        .map_err(|_| "clipboard から読み取れません".to_string())?;
    value
        .as_string()
        .ok_or_else(|| "clipboard が空です".to_string())
}

#[cfg(not(feature = "web"))]
pub async fn read_from_clipboard() -> Result<String, String> {
    Err("web でのみ貼り付けできます".to_string())
}
