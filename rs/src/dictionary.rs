use crate::models::{Dictionary, TagSuggestion};
use crate::utils::{format_description_with_suffix, utf8_has_multibyte};
use std::collections::{HashMap, HashSet};

const DICTIONARY_METADATA: &str = include_str!("../assets/danbooru-machine-jp.csv");
const DICTIONARY_CATEGORY: &str = include_str!("../assets/danbooru.csv");

pub fn load_dictionary() -> Result<Dictionary, String> {
    let mut metadata = HashMap::new();
    let mut categories = HashMap::new();
    let mut order = Vec::new();
    let mut order_lower = Vec::new();
    let mut prefix_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut metadata_scan: Vec<(String, usize)> = Vec::new();

    for line in DICTIONARY_METADATA.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, ',');
        let raw_tag = parts.next().unwrap_or_default();
        let meta = parts.next().unwrap_or_default();
        if raw_tag.is_empty() || meta.is_empty() {
            continue;
        }
        let tag = booru_to_image_tag(raw_tag.trim());
        let lower = tag.to_ascii_lowercase();
        metadata.insert(lower.clone(), meta.trim().to_string());
        order.push(tag.clone());
        order_lower.push(lower.clone());
        let idx = order.len() - 1;
        metadata_scan.push((meta.trim().to_lowercase(), idx));
        for len in 1..=3 {
            let prefix: String = lower.chars().take(len).collect();
            if prefix.is_empty() {
                continue;
            }
            prefix_index.entry(prefix).or_default().push(idx);
        }
    }

    for line in DICTIONARY_CATEGORY.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, ',');
        let raw_tag = parts.next().unwrap_or_default();
        let cat = parts.next().unwrap_or_default();
        if raw_tag.is_empty() || cat.is_empty() {
            continue;
        }
        if let Ok(category) = cat.trim().parse::<i32>() {
            let tag = booru_to_image_tag(raw_tag.trim()).to_ascii_lowercase();
            categories.insert(tag, category);
        }
    }

    if metadata.is_empty() || categories.is_empty() {
        return Err("辞書ファイルを読み込めませんでした".to_string());
    }

    let mut order_map = HashMap::new();
    for (idx, tag) in order.iter().enumerate() {
        order_map.insert(tag.to_ascii_lowercase(), idx);
    }

    Ok(Dictionary {
        metadata,
        categories,
        order,
        order_map,
        order_lower,
        metadata_scan,
        prefix_index,
    })
}

pub fn booru_to_image_tag(input: &str) -> String {
    input
        .replace('_', " ")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub fn generate_suggestions(input: &str, dict: &Dictionary) -> Vec<TagSuggestion> {
    let query = input.trim();
    if query.is_empty() {
        return Vec::new();
    }

    if utf8_has_multibyte(query) {
        return metadata_reverse_search(query, dict, 40);
    }

    let prefix_limit = 8;
    let mut suggestions = prefix_suggestions(query, dict, prefix_limit);
    let mut seen: HashSet<String> = suggestions
        .iter()
        .map(|s| s.tag.to_ascii_lowercase())
        .collect();

    if suggestions.len() >= prefix_limit {
        return suggestions;
    }

    for item in fuzzy_suggestions(query, dict, 32) {
        if seen.insert(item.tag.to_ascii_lowercase()) {
            suggestions.push(item.clone());
        }
    }
    suggestions
}

fn prefix_suggestions(query: &str, dict: &Dictionary, limit: usize) -> Vec<TagSuggestion> {
    let mut results = Vec::new();
    let lower = query.to_ascii_lowercase();

    let max_prefix = lower.chars().count().min(3);
    for len in (1..=max_prefix).rev() {
        let prefix: String = lower.chars().take(len).collect();
        if let Some(indices) = dict.prefix_index.get(&prefix) {
            for &idx in indices.iter() {
                if results.len() >= limit {
                    break;
                }
                if let (Some(orig), Some(lower_tag)) =
                    (dict.order.get(idx), dict.order_lower.get(idx))
                {
                    if lower_tag.starts_with(&lower) {
                        results.push(dict.make_suggestion(orig));
                    }
                }
            }
            if !results.is_empty() || len == 1 {
                break;
            }
        }
    }

    if results.is_empty() && lower.len() <= 1 {
        for (orig, lower_tag) in dict.order.iter().zip(dict.order_lower.iter()) {
            if results.len() >= limit {
                break;
            }
            if lower_tag.starts_with(&lower) {
                results.push(dict.make_suggestion(orig));
            }
        }
    }
    results
}

fn fuzzy_suggestions(query: &str, dict: &Dictionary, limit: usize) -> Vec<TagSuggestion> {
    let lower = query.to_ascii_lowercase();
    let qlen = lower.len();
    let max_scan = dict.order_lower.len().min(4000);
    let mut scored: Vec<(f64, usize)> = Vec::new();

    let candidates: Vec<usize> = if let Some(first) = lower.chars().next() {
        let key = first.to_string();
        dict.prefix_index
            .get(&key)
            .map(|indices| indices.iter().copied().take(max_scan).collect())
            .unwrap_or_else(|| (0..max_scan).collect())
    } else {
        (0..max_scan).collect()
    };

    for idx in candidates {
        let tag = match dict.order_lower.get(idx) {
            Some(t) => t,
            None => continue,
        };
        let tlen = tag.len();
        if qlen > tlen + 8 || tlen > qlen + 8 {
            continue;
        }
        if !tag.contains(&lower) && !tag.starts_with(lower.chars().next().unwrap_or_default()) {
            continue;
        }
        let score = if tag.contains(&lower) {
            1.0
        } else {
            strsim::jaro_winkler(&lower, tag)
        };
        if score >= 0.82 {
            scored.push((score, idx));
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, idx)| dict.make_suggestion(&dict.order[idx]))
        .collect()
}

fn metadata_reverse_search(query: &str, dict: &Dictionary, limit: usize) -> Vec<TagSuggestion> {
    let mut results = Vec::new();
    let lower = query.to_lowercase();

    let mut scanned = 0usize;
    for (meta_lower, idx) in dict.metadata_scan.iter() {
        if results.len() >= limit {
            break;
        }
        scanned += 1;
        let len_diff = meta_lower.len().abs_diff(lower.len());
        if !meta_lower.contains(&lower) && len_diff > 8 {
            continue;
        }
        let score = if meta_lower.contains(&lower) {
            1.0
        } else {
            strsim::jaro_winkler(meta_lower, &lower)
        };
        if score >= 0.75 {
            if let Some(tag) = dict.order.get(*idx) {
                results.push(dict.make_suggestion(tag));
            }
        }
        if scanned >= 6000 && results.is_empty() {
            // avoid scanning the full dictionary on very long queries that won't match
            break;
        }
    }
    results
}

impl Dictionary {
    pub fn make_suggestion(&self, tag: &str) -> TagSuggestion {
        let key = tag.to_ascii_lowercase();
        let category = self.categories.get(&key).copied().unwrap_or(0);
        let description = self
            .metadata
            .get(&key)
            .map(|d| format_description_with_suffix(d, category));
        TagSuggestion {
            tag: tag.to_string(),
            description,
            category,
        }
    }

    pub fn order_index(&self, tag: &str) -> usize {
        self.order_map
            .get(tag)
            .copied()
            .unwrap_or(self.order.len() + 1)
    }
}
