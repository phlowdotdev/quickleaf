//! List properties for configuring cache query behavior.
//!
//! This module provides structures and enums for configuring how cache entries
//! are retrieved, ordered, filtered, and paginated.

use crate::filter::Filter;

/// Enum for specifying sort order when listing cache entries.
///
/// # Examples
///
/// ```
/// use quickleaf::Order;
/// use quickleaf::Cache;
/// use quickleaf::ListProps;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(10);
/// cache.insert("zebra", 1);
/// cache.insert("apple", 2);
/// cache.insert("monkey", 3);
///
/// // Ascending order (default)
/// let props = ListProps::default().order(Order::Asc);
/// let results = cache.list(props).unwrap();
/// let keys: Vec<_> = results.iter().map(|(k, _)| k.as_str()).collect();
/// assert_eq!(keys, vec!["apple", "monkey", "zebra"]);
///
/// // Descending order
/// let props = ListProps::default().order(Order::Desc);
/// let results = cache.list(props).unwrap();
/// let keys: Vec<_> = results.iter().map(|(k, _)| k.as_str()).collect();
/// assert_eq!(keys, vec!["zebra", "monkey", "apple"]);
/// ```
#[derive(Debug, Clone)]
pub enum Order {
    /// Sort keys in ascending order (A-Z).
    Asc,
    /// Sort keys in descending order (Z-A).
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Self::Asc
    }
}

/// Enum for specifying pagination starting point.
///
/// # Examples
///
/// ```
/// use quickleaf::{StartAfter, ListProps, Order};
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(10);
/// cache.insert("apple", 1);
/// cache.insert("banana", 2);
/// cache.insert("cherry", 3);
///
/// // Start listing after "banana"
/// let props = ListProps::default()
///     .start_after_key("banana")
///     .order(Order::Asc);
/// let results = cache.list(props).unwrap();
/// let keys: Vec<_> = results.iter().map(|(k, _)| k.as_str()).collect();
/// assert_eq!(keys, vec!["cherry"]);
/// ```
#[derive(Debug, Clone)]
pub enum StartAfter {
    /// Start listing after the specified key.
    Key(String),
    /// Start from the beginning.
    None,
}

impl Default for StartAfter {
    fn default() -> Self {
        Self::None
    }
}

/// Configuration structure for listing cache entries with filtering, ordering, and pagination.
///
/// `ListProps` allows you to customize how cache entries are retrieved:
/// - Filter by key patterns
/// - Sort in ascending or descending order
/// - Paginate results with starting points and limits
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use quickleaf::{ListProps, Order};
/// use quickleaf::Filter;
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(10);
/// cache.insert("apple", 1);
/// cache.insert("banana", 2);
/// cache.insert("apricot", 3);
///
/// let props = ListProps::default()
///     .order(Order::Desc)
///     .filter(Filter::StartWith("ap".to_string()));
///
/// let results = cache.list(props).unwrap();
/// assert_eq!(results.len(), 2);
/// ```
///
/// ## Pagination
///
/// ```
/// use quickleaf::ListProps;
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(20);
/// for i in 0..20 {
///     cache.insert(format!("key_{:02}", i), i);
/// }
///
/// // Get first page (default limit is 10)
/// let props = ListProps::default();
/// let page1 = cache.list(props).unwrap();
/// assert_eq!(page1.len(), 10);
///
/// // Get next page starting after the last key from page1
/// let last_key = &page1.last().unwrap().0;
/// let props = ListProps::default().start_after_key(last_key);
/// let page2 = cache.list(props).unwrap();
/// assert_eq!(page2.len(), 10);
/// ```
#[derive(Debug)]
pub struct ListProps {
    /// Starting point for pagination.
    pub start_after_key: StartAfter,
    /// Filter to apply to keys.
    pub filter: Filter,
    /// Sort order for results.
    pub order: Order,
    /// Maximum number of results to return.
    pub limit: usize,
}

impl Default for ListProps {
    fn default() -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }
}

impl ListProps {
    /// Creates a new `ListProps` with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::ListProps;
    ///
    /// let props = ListProps::default();
    /// // Equivalent to creating with default values
    /// ```
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }

    /// Sets the starting point for pagination.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::ListProps;
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("apple", 1);
    /// cache.insert("banana", 2);
    /// cache.insert("cherry", 3);
    ///
    /// let props = ListProps::default().start_after_key("banana");
    /// let results = cache.list(props).unwrap();
    /// // Will return entries after "banana"
    /// ```
    pub fn start_after_key(mut self, key: &str) -> Self {
        self.start_after_key = StartAfter::Key(key.to_string());
        self
    }

    /// Sets the filter for key matching.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::ListProps;
    /// use quickleaf::Filter;
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("user_123", 1);
    /// cache.insert("user_456", 2);
    /// cache.insert("admin_789", 3);
    ///
    /// let props = ListProps::default()
    ///     .filter(Filter::StartWith("user_".to_string()));
    /// let results = cache.list(props).unwrap();
    /// assert_eq!(results.len(), 2);
    /// ```
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the sort order for results.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::{ListProps, Order};
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("zebra", 1);
    /// cache.insert("apple", 2);
    ///
    /// let props = ListProps::default().order(Order::Desc);
    /// let results = cache.list(props).unwrap();
    /// let keys: Vec<_> = results.iter().map(|(k, _)| k.as_str()).collect();
    /// assert_eq!(keys, vec!["zebra", "apple"]);
    /// ```
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }

    /// Sets the maximum number of results to return.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::ListProps;
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(20);
    /// for i in 0..15 {
    ///     cache.insert(format!("key_{:02}", i), i);
    /// }
    ///
    /// // Limit results to 5 items
    /// let props = ListProps::default().limit(5);
    /// let results = cache.list(props).unwrap();
    /// assert_eq!(results.len(), 5);
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl From<Filter> for ListProps {
    fn from(filter: Filter) -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter,
            order: Order::Asc,
            limit: 10,
        }
    }
}

impl From<Order> for ListProps {
    fn from(order: Order) -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order,
            limit: 10,
        }
    }
}

impl From<StartAfter> for ListProps {
    fn from(start_after_key: StartAfter) -> Self {
        Self {
            start_after_key,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }
}
