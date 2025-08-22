//! Error types for cache operations.
//!
//! This module defines the error types that can occur during cache operations.

use std::fmt::{Debug, Display};

/// Errors that can occur during cache operations.
///
/// # Examples
///
/// ```
/// use quickleaf::Error;
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(10);
/// 
/// // Trying to remove a non-existent key returns an error
/// match cache.remove("nonexistent") {
///     Err(Error::KeyNotFound) => println!("Key not found as expected"),
///     _ => panic!("Expected KeyNotFound error"),
/// }
/// ```
#[derive(PartialEq)]
pub enum Error {
    /// The specified sort key was not found during list operations.
    ///
    /// This can occur when using `start_after_key` in `ListProps` with a key
    /// that doesn't exist in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Error;
    /// use quickleaf::Cache;
    /// use quickleaf::ListProps;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("existing_key", "value");
    ///
    /// let props = ListProps::default().start_after_key("nonexistent_key");
    /// match cache.list(props) {
    ///     Err(Error::SortKeyNotFound) => println!("Sort key not found"),
    ///     _ => panic!("Expected SortKeyNotFound error"),
    /// }
    /// ```
    SortKeyNotFound,
    
    /// A cache with the same identifier already exists.
    ///
    /// This error is currently not used in the main API but reserved for
    /// future functionality.
    CacheAlreadyExists,
    
    /// A sort key already exists.
    ///
    /// This error is currently not used in the main API but reserved for
    /// future functionality.
    SortKeyExists,
    
    /// A table with the same name already exists.
    ///
    /// This error is currently not used in the main API but reserved for
    /// future functionality.
    TableAlreadyExists,
    
    /// The specified key was not found in the cache.
    ///
    /// This occurs when trying to remove a key that doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Error;
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// 
    /// match cache.remove("missing_key") {
    ///     Err(Error::KeyNotFound) => println!("Key not found"),
    ///     Err(_) => println!("Other error"),
    ///     Ok(_) => panic!("Expected an error"),
    /// }
    /// ```
    KeyNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SortKeyNotFound => write!(f, "Sort key not found"),
            Error::CacheAlreadyExists => write!(f, "Cache already exists"),
            Error::SortKeyExists => write!(f, "Sort key exists"),
            Error::TableAlreadyExists => write!(f, "Table already exists"),
            Error::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl std::error::Error for Error {}
