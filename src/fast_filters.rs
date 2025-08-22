//! Optimized filter operations for better performance

use crate::filter::Filter;

/// Fast prefix matching using byte-level operations
#[inline]
pub fn fast_prefix_match(text: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if text.len() < prefix.len() {
        return false;
    }

    unsafe {
        let text_bytes = text.as_bytes();
        let prefix_bytes = prefix.as_bytes();

        let chunks = prefix_bytes.len() / 8;
        let mut i = 0;

        for _ in 0..chunks {
            let text_chunk = std::ptr::read_unaligned(text_bytes.as_ptr().add(i) as *const u64);
            let prefix_chunk = std::ptr::read_unaligned(prefix_bytes.as_ptr().add(i) as *const u64);

            if text_chunk != prefix_chunk {
                return false;
            }
            i += 8;
        }

        for j in i..prefix_bytes.len() {
            if text_bytes[j] != prefix_bytes[j] {
                return false;
            }
        }
    }

    true
}

/// Fast suffix matching optimized for common cases
#[inline]
pub fn fast_suffix_match(text: &str, suffix: &str) -> bool {
    if suffix.is_empty() {
        return true;
    }
    if text.len() < suffix.len() {
        return false;
    }

    let text_bytes = text.as_bytes();
    let suffix_bytes = suffix.as_bytes();
    let start_pos = text_bytes.len() - suffix_bytes.len();

    unsafe {
        let text_suffix = text_bytes.as_ptr().add(start_pos);
        let suffix_ptr = suffix_bytes.as_ptr();

        libc::memcmp(
            text_suffix as *const libc::c_void,
            suffix_ptr as *const libc::c_void,
            suffix_bytes.len(),
        ) == 0
    }
}

/// Optimized filter application
#[inline]
pub fn apply_filter_fast(key: &str, filter: &Filter) -> bool {
    match filter {
        Filter::None => true,
        Filter::StartWith(prefix) => fast_prefix_match(key, prefix),
        Filter::EndWith(suffix) => fast_suffix_match(key, suffix),
        Filter::StartAndEndWith(prefix, suffix) => {
            fast_prefix_match(key, prefix) && fast_suffix_match(key, suffix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
