use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    Stacked,
}

impl LayoutMode {
    pub fn class(&self) -> &'static str {
        match self {
            LayoutMode::Stacked => "stack-layout",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortMode {
    Original,
    Alphabetical,
    Favorite,
    Category,
    Custom,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Original => "Original",
            SortMode::Alphabetical => "A-Z",
            SortMode::Favorite => "Fav",
            SortMode::Category => "Category",
            SortMode::Custom => "Custom",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagEntry {
    pub text: String,
    pub description: Option<String>,
    pub category: i32,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagSuggestion {
    pub tag: String,
    pub description: Option<String>,
    pub category: i32,
}

#[derive(Clone)]
pub struct Dictionary {
    pub metadata: HashMap<String, String>,
    pub categories: HashMap<String, i32>,
    pub order: Vec<String>,
    pub order_map: HashMap<String, usize>,
    pub order_lower: Vec<String>,
    pub metadata_scan: Vec<(String, usize)>,
    pub prefix_index: HashMap<String, Vec<usize>>,
}

#[derive(Clone)]
pub enum DictionaryState {
    Loading,
    Ready(Arc<Dictionary>),
    Failed(String),
}

#[derive(Clone, Copy)]
pub enum TagMove {
    Up,
    Down,
    Top,
    Bottom,
}
