use crate::models::{Dictionary, TagEntry};
use crate::utils::format_description_with_suffix;

pub fn extract_tags(text: &str) -> Vec<TagEntry> {
    let mut tags = Vec::new();
    let mut token_start = 0usize;

    for (idx, ch) in text.char_indices() {
        if ch == ',' || ch == '\n' {
            push_token(text, token_start, idx, &mut tags);
            if ch == '\n' {
                tags.push(TagEntry {
                    text: "\n".to_string(),
                    description: None,
                    category: 0,
                    start: idx,
                    end: idx + ch.len_utf8(),
                });
            }
            token_start = idx + ch.len_utf8();
        }
    }

    if token_start <= text.len() {
        push_token(text, token_start, text.len(), &mut tags);
    }

    tags
}

fn push_token(text: &str, start: usize, end: usize, out: &mut Vec<TagEntry>) {
    if start >= end {
        return;
    }
    let slice = &text[start..end];
    let trimmed_start = slice.len() - slice.trim_start().len();
    let trimmed_end = slice.len() - slice.trim_end().len();
    let actual_start = start + trimmed_start;
    let actual_end = end - trimmed_end;
    if actual_start < actual_end {
        out.push(TagEntry {
            text: text[actual_start..actual_end].to_string(),
            description: None,
            category: 0,
            start: actual_start,
            end: actual_end,
        });
    }
}

pub fn decorate_tags(tags: &mut [TagEntry], dict: &Dictionary) {
    for tag in tags.iter_mut() {
        if tag.text == "\n" {
            continue;
        }
        let key = tag.text.to_ascii_lowercase();
        if let Some(category) = dict.categories.get(&key) {
            tag.category = *category;
        }
        if let Some(desc) = dict.metadata.get(&key) {
            tag.description = Some(format_description_with_suffix(desc, tag.category));
        }
    }
}

pub fn rebuild_prompt_from_tags(tags: &[TagEntry]) -> String {
    let mut output = String::new();
    let mut is_first = true;
    for tag in tags {
        if tag.text == "\n" {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            is_first = true;
            continue;
        }
        if !is_first {
            output.push_str(", ");
        }
        output.push_str(&tag.text);
        is_first = false;
    }
    output
}
