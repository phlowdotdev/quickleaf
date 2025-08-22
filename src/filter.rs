//! Filtering functionality for cache queries.
//!
//! This module provides different types of filters that can be applied when listing cache entries.
//! Filters allow you to narrow down results based on key patterns.

/// Enum representing different filter types for cache queries.
///
/// Filters are used with the `list` method to narrow down results based on key patterns.
///
/// # Examples
///
/// ```
/// use quickleaf::Filter;
/// use quickleaf::Cache;
/// use quickleaf::ListProps;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(10);
/// cache.insert("apple_pie", 1);
/// cache.insert("banana_split", 2);
/// cache.insert("apple_juice", 3);
/// cache.insert("grape_juice", 4);
///
/// // Filter by prefix
/// let start_filter = Filter::StartWith("apple".to_string());
/// let props = ListProps::default().filter(start_filter);
/// let results = cache.list(props).unwrap();
/// assert_eq!(results.len(), 2);
///
/// // Filter by suffix
/// let end_filter = Filter::EndWith("juice".to_string());
/// let props = ListProps::default().filter(end_filter);
/// let results = cache.list(props).unwrap();
/// assert_eq!(results.len(), 2);
///
/// // Filter by both prefix and suffix
/// let both_filter = Filter::StartAndEndWith("apple".to_string(), "juice".to_string());
/// let props = ListProps::default().filter(both_filter);
/// let results = cache.list(props).unwrap();
/// assert_eq!(results.len(), 1);
/// ```
#[derive(Debug)]
pub enum Filter {
    /// Filter keys that start with the specified string.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Filter;
    ///
    /// let filter = Filter::StartWith("user_".to_string());
    /// // This will match keys like "user_123", "user_session", etc.
    /// ```
    StartWith(String),

    /// Filter keys that end with the specified string.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Filter;
    ///
    /// let filter = Filter::EndWith("_cache".to_string());
    /// // This will match keys like "session_cache", "user_cache", etc.
    /// ```
    EndWith(String),

    /// Filter keys that start with the first string AND end with the second string.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Filter;
    ///
    /// let filter = Filter::StartAndEndWith("temp_".to_string(), "_data".to_string());
    /// // This will match keys like "temp_session_data", "temp_user_data", etc.
    /// ```
    StartAndEndWith(String, String),

    /// No filtering applied - returns all items.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Filter;
    ///
    /// let filter = Filter::None;
    /// // This will return all cache entries
    /// ```
    None,
}

impl Default for Filter {
    fn default() -> Self {
        Self::None
    }
}
