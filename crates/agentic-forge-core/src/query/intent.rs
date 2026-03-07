//! Intent declaration and scoped extraction.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionIntent {
    Exists,
    IdsOnly,
    Summary,
    Fields(Vec<String>),
    Full,
}

impl ExtractionIntent {
    pub fn estimated_tokens(&self) -> u64 {
        match self {
            Self::Exists => 1,
            Self::IdsOnly => 10,
            Self::Summary => 50,
            Self::Fields(f) => 20 * f.len() as u64,
            Self::Full => 500,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "exists" => Self::Exists,
            "ids" | "ids_only" => Self::IdsOnly,
            "summary" => Self::Summary,
            "full" => Self::Full,
            _ => Self::Full,
        }
    }

    pub fn is_minimal(&self) -> bool {
        matches!(self, Self::Exists | Self::IdsOnly)
    }

    pub fn includes_content(&self) -> bool {
        matches!(self, Self::Summary | Self::Fields(_) | Self::Full)
    }
}

impl Default for ExtractionIntent {
    fn default() -> Self {
        Self::IdsOnly // Token-conservative default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopedResult {
    Bool(bool),
    Id(String),
    Ids(Vec<String>),
    Summary(String),
    Fields(HashMap<String, Value>),
    Full(Value),
    Count(usize),
}

impl ScopedResult {
    pub fn estimated_tokens(&self) -> u64 {
        match self {
            Self::Bool(_) => 1,
            Self::Id(_) => 5,
            Self::Ids(ids) => ids.len() as u64 * 5,
            Self::Summary(_) => 50,
            Self::Fields(f) => f.len() as u64 * 20,
            Self::Full(v) => serde_json::to_string(v).map(|s| s.len() as u64 / 4).unwrap_or(500),
            Self::Count(_) => 2,
        }
    }
}

pub trait Scopeable {
    fn id_str(&self) -> String;
    fn summarize(&self) -> String;
    fn extract_fields(&self, fields: &[String]) -> HashMap<String, Value>;
    fn to_json(&self) -> Value;
}

pub fn apply_intent<T: Scopeable>(intent: &ExtractionIntent, item: &T) -> ScopedResult {
    match intent {
        ExtractionIntent::Exists => ScopedResult::Bool(true),
        ExtractionIntent::IdsOnly => ScopedResult::Id(item.id_str()),
        ExtractionIntent::Summary => ScopedResult::Summary(item.summarize()),
        ExtractionIntent::Fields(f) => ScopedResult::Fields(item.extract_fields(f)),
        ExtractionIntent::Full => ScopedResult::Full(item.to_json()),
    }
}

pub fn apply_intent_many<T: Scopeable>(intent: &ExtractionIntent, items: &[T]) -> ScopedResult {
    match intent {
        ExtractionIntent::Exists => ScopedResult::Bool(!items.is_empty()),
        ExtractionIntent::IdsOnly => ScopedResult::Ids(items.iter().map(|i| i.id_str()).collect()),
        ExtractionIntent::Summary => ScopedResult::Count(items.len()),
        _ => ScopedResult::Full(serde_json::json!(items.iter().map(|i| i.to_json()).collect::<Vec<_>>())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockItem { id: String, name: String }
    impl Scopeable for MockItem {
        fn id_str(&self) -> String { self.id.clone() }
        fn summarize(&self) -> String { format!("{}: {}", self.id, self.name) }
        fn extract_fields(&self, fields: &[String]) -> HashMap<String, Value> {
            let mut map = HashMap::new();
            for f in fields {
                match f.as_str() {
                    "name" => { map.insert("name".into(), Value::String(self.name.clone())); }
                    _ => {}
                }
            }
            map
        }
        fn to_json(&self) -> Value { serde_json::json!({"id": self.id, "name": self.name}) }
    }

    #[test]
    fn test_intent_exists() {
        let item = MockItem { id: "1".into(), name: "Test".into() };
        let result = apply_intent(&ExtractionIntent::Exists, &item);
        assert!(matches!(result, ScopedResult::Bool(true)));
        assert_eq!(result.estimated_tokens(), 1);
    }

    #[test]
    fn test_intent_ids_only() {
        let item = MockItem { id: "42".into(), name: "Test".into() };
        let result = apply_intent(&ExtractionIntent::IdsOnly, &item);
        assert!(matches!(result, ScopedResult::Id(ref s) if s == "42"));
    }

    #[test]
    fn test_intent_summary() {
        let item = MockItem { id: "1".into(), name: "Test".into() };
        let result = apply_intent(&ExtractionIntent::Summary, &item);
        assert!(matches!(result, ScopedResult::Summary(_)));
    }

    #[test]
    fn test_intent_fields() {
        let item = MockItem { id: "1".into(), name: "Hello".into() };
        let result = apply_intent(&ExtractionIntent::Fields(vec!["name".into()]), &item);
        if let ScopedResult::Fields(map) = result {
            assert_eq!(map.get("name"), Some(&Value::String("Hello".into())));
        } else { panic!("Expected Fields"); }
    }

    #[test]
    fn test_intent_full() {
        let item = MockItem { id: "1".into(), name: "Full".into() };
        let result = apply_intent(&ExtractionIntent::Full, &item);
        assert!(matches!(result, ScopedResult::Full(_)));
    }

    #[test]
    fn test_intent_default_is_minimal() {
        let intent = ExtractionIntent::default();
        assert!(intent.is_minimal());
        assert!(!intent.includes_content());
    }

    #[test]
    fn test_scoped_query_cheaper_than_full() {
        let ids_cost = ExtractionIntent::IdsOnly.estimated_tokens();
        let full_cost = ExtractionIntent::Full.estimated_tokens();
        assert!(ids_cost < full_cost / 10, "IDs should be >10x cheaper than Full");
    }

    #[test]
    fn test_intent_from_str() {
        assert_eq!(ExtractionIntent::from_str("exists"), ExtractionIntent::Exists);
        assert_eq!(ExtractionIntent::from_str("ids"), ExtractionIntent::IdsOnly);
        assert_eq!(ExtractionIntent::from_str("summary"), ExtractionIntent::Summary);
        assert_eq!(ExtractionIntent::from_str("full"), ExtractionIntent::Full);
        assert_eq!(ExtractionIntent::from_str("unknown"), ExtractionIntent::Full);
    }

    #[test]
    fn test_apply_intent_many_ids() {
        let items = vec![
            MockItem { id: "1".into(), name: "A".into() },
            MockItem { id: "2".into(), name: "B".into() },
        ];
        let result = apply_intent_many(&ExtractionIntent::IdsOnly, &items);
        if let ScopedResult::Ids(ids) = result {
            assert_eq!(ids, vec!["1", "2"]);
        } else { panic!("Expected Ids"); }
    }

    #[test]
    fn test_apply_intent_many_exists_empty() {
        let items: Vec<MockItem> = vec![];
        let result = apply_intent_many(&ExtractionIntent::Exists, &items);
        assert!(matches!(result, ScopedResult::Bool(false)));
    }
}
