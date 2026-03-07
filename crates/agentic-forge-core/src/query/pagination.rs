//! Cursor-based pagination.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub total: Option<usize>,
}

impl<T> CursorPage<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            cursor: None,
            has_more: false,
            total: Some(0),
        }
    }

    pub fn from_slice(all: Vec<T>, cursor: Option<&str>, limit: usize) -> Self
    where
        T: Clone,
    {
        let start = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
        let total = all.len();

        if start >= total {
            return Self {
                items: Vec::new(),
                cursor: None,
                has_more: false,
                total: Some(total),
            };
        }

        let end = (start + limit).min(total);
        let items = all[start..end].to_vec();
        let has_more = end < total;
        let next_cursor = if has_more {
            Some(end.to_string())
        } else {
            None
        };

        Self {
            items,
            cursor: next_cursor,
            has_more,
            total: Some(total),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_page_first() {
        let data: Vec<i32> = (0..20).collect();
        let page = CursorPage::from_slice(data, None, 5);
        assert_eq!(page.len(), 5);
        assert!(page.has_more);
        assert_eq!(page.cursor, Some("5".into()));
        assert_eq!(page.total, Some(20));
    }

    #[test]
    fn test_cursor_page_second() {
        let data: Vec<i32> = (0..20).collect();
        let page = CursorPage::from_slice(data, Some("5"), 5);
        assert_eq!(page.items, vec![5, 6, 7, 8, 9]);
        assert!(page.has_more);
    }

    #[test]
    fn test_cursor_page_last() {
        let data: Vec<i32> = (0..20).collect();
        let page = CursorPage::from_slice(data, Some("15"), 5);
        assert_eq!(page.len(), 5);
        assert!(!page.has_more);
        assert_eq!(page.cursor, None);
    }

    #[test]
    fn test_cursor_page_past_end() {
        let data: Vec<i32> = (0..5).collect();
        let page = CursorPage::from_slice(data, Some("100"), 5);
        assert!(page.is_empty());
        assert!(!page.has_more);
    }

    #[test]
    fn test_cursor_page_empty() {
        let page: CursorPage<i32> = CursorPage::empty();
        assert!(page.is_empty());
        assert_eq!(page.total, Some(0));
    }

    #[test]
    fn test_cursor_page_exact_boundary() {
        let data: Vec<i32> = (0..10).collect();
        let page = CursorPage::from_slice(data, None, 10);
        assert_eq!(page.len(), 10);
        assert!(!page.has_more);
    }

    #[test]
    fn test_cursor_page_limit_larger_than_data() {
        let data: Vec<i32> = (0..3).collect();
        let page = CursorPage::from_slice(data, None, 100);
        assert_eq!(page.len(), 3);
        assert!(!page.has_more);
    }
}
