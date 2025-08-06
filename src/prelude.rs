//! Prelude module for convenient imports.
//!
//! This module re-exports commonly used items to make them easier to import.
//! Import this module to get access to the most frequently used traits and types.

/// Re-exports from the valu3 library for convenient access to value conversion traits.
///
/// # Examples
///
/// ```
/// use quickleaf::prelude::*;
/// use quickleaf::Quickleaf;
///
/// let mut cache = Quickleaf::new(10);
/// 
/// // ToValueBehavior trait is available from the prelude
/// cache.insert("number", 42);
/// cache.insert("string", "hello");
/// cache.insert("boolean", true);
/// 
/// assert_eq!(cache.get("number"), Some(&42.to_value()));
/// assert_eq!(cache.get("string"), Some(&"hello".to_value()));
/// assert_eq!(cache.get("boolean"), Some(&true.to_value()));
/// ```
pub use valu3::prelude::*;
