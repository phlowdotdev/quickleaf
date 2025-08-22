//! Optimized filter operations - simple and fast!

use crate::filter::Filter;

/// Fast and safe prefix matching using Rust's optimized implementation
#[inline(always)]
pub fn fast_prefix_match(text: &str, prefix: &str) -> bool {
    text.starts_with(prefix)
}

/// Fast and safe suffix matching using Rust's optimized implementation  
#[inline(always)]
pub fn fast_suffix_match(text: &str, suffix: &str) -> bool {
    text.ends_with(suffix)
}

/// Optimized filter application - same interface, better performance
#[inline]
pub fn apply_filter_fast(key: &str, filter: &Filter) -> bool {
    match filter {
        Filter::None => true,
        Filter::StartWith(prefix) => key.starts_with(prefix),
        Filter::EndWith(suffix) => key.ends_with(suffix),
        Filter::StartAndEndWith(prefix, suffix) => key.starts_with(prefix) && key.ends_with(suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::Filter;

    #[test]
    fn test_fast_prefix_match() {
        assert!(fast_prefix_match("hello_world", "hello"));
        assert!(fast_prefix_match("hello", "hello"));
        assert!(!fast_prefix_match("hello", "hello_world"));
        assert!(fast_prefix_match("test", ""));
    }

    #[test]
    fn test_fast_suffix_match() {
        assert!(fast_suffix_match("hello_world", "world"));
        assert!(fast_suffix_match("world", "world"));
        assert!(!fast_suffix_match("world", "hello_world"));
        assert!(fast_suffix_match("test", ""));
    }

    #[test]
    fn test_apply_filter_fast() {
        assert!(apply_filter_fast("test", &Filter::None));
        assert!(apply_filter_fast(
            "hello_world",
            &Filter::StartWith("hello".to_string())
        ));
        assert!(apply_filter_fast(
            "hello_world",
            &Filter::EndWith("world".to_string())
        ));
        assert!(apply_filter_fast(
            "hello_world",
            &Filter::StartAndEndWith("hello".to_string(), "world".to_string())
        ));
        assert!(!apply_filter_fast(
            "hello_world",
            &Filter::StartWith("goodbye".to_string())
        ));
    }
}
