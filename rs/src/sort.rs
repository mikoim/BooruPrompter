use crate::models::{Dictionary, SortMode, TagEntry};

pub fn sort_to_string(mode: SortMode) -> String {
    match mode {
        SortMode::Original => "original".to_string(),
        SortMode::Alphabetical => "alphabetical".to_string(),
        SortMode::Favorite => "favorite".to_string(),
        SortMode::Category => "category".to_string(),
        SortMode::Custom => "custom".to_string(),
    }
}

pub fn sort_from_string(s: &str) -> SortMode {
    match s {
        "alphabetical" => SortMode::Alphabetical,
        "favorite" => SortMode::Favorite,
        "category" => SortMode::Category,
        "custom" => SortMode::Custom,
        _ => SortMode::Original,
    }
}

pub fn sort_tags(tags: &[TagEntry], mode: SortMode, dict: Option<&Dictionary>) -> Vec<TagEntry> {
    match mode {
        SortMode::Original => tags.to_vec(),
        SortMode::Alphabetical => sort_within_segments(tags, |mut segment| {
            segment.sort_by_key(|t| t.text.to_ascii_lowercase());
            segment
        }),
        SortMode::Favorite => sort_within_segments(tags, |segment| {
            if let Some(dict) = dict {
                // Pre-compute lowercase keys to avoid O(n log n) conversions
                let mut indexed: Vec<_> = segment
                    .into_iter()
                    .map(|t| {
                        let lower = t.text.to_ascii_lowercase();
                        (dict.order_index(&lower), t)
                    })
                    .collect();
                indexed.sort_by_key(|(idx, _)| *idx);
                indexed.into_iter().map(|(_, t)| t).collect()
            } else {
                segment
            }
        }),
        SortMode::Category => sort_within_segments(tags, |segment| {
            if let Some(dict) = dict {
                // Pre-compute lowercase keys to avoid O(n log n) conversions
                let mut indexed: Vec<_> = segment
                    .into_iter()
                    .map(|t| {
                        let lower = t.text.to_ascii_lowercase();
                        let cat = dict.categories.get(&lower).copied().unwrap_or(0);
                        let order = dict.order_index(&lower);
                        (cat, order, t)
                    })
                    .collect();
                indexed.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                indexed.into_iter().map(|(_, _, t)| t).collect()
            } else {
                segment
            }
        }),
        SortMode::Custom => sort_within_segments(tags, custom_sort_segment),
    }
}

pub fn sort_within_segments(
    tags: &[TagEntry],
    sorter: impl Fn(Vec<TagEntry>) -> Vec<TagEntry>,
) -> Vec<TagEntry> {
    let mut current = Vec::new();
    let mut output = Vec::new();
    for tag in tags {
        if tag.text == "\n" {
            output.extend(sorter(current));
            output.push(tag.clone());
            current = Vec::new();
        } else {
            current.push(tag.clone());
        }
    }
    output.extend(sorter(current));
    output
}

pub fn custom_sort_segment(tags: Vec<TagEntry>) -> Vec<TagEntry> {
    if tags.len() < 2 {
        return tags;
    }
    let mut groups: Vec<(String, Vec<TagEntry>)> = Vec::new();
    for tag in tags.into_iter() {
        let last_word = tag.text.split_whitespace().last().unwrap_or("").to_string();
        if let Some((_, items)) = groups.iter_mut().find(|(key, _)| *key == last_word) {
            items.push(tag);
        } else {
            groups.push((last_word, vec![tag]));
        }
    }

    let mut output = Vec::new();
    for (_, items) in groups {
        if items.is_empty() {
            continue;
        }
        let mut filtered: Vec<_> = if items.len() > 1 {
            items
                .into_iter()
                .filter(|t| t.text.split_whitespace().count() > 1)
                .collect()
        } else {
            items
        };
        if filtered.is_empty() {
            continue;
        }
        output.append(&mut filtered);
    }
    output
}
