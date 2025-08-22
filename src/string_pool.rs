use hashbrown::HashMap;
use std::sync::Arc;

/// String pool para reutilizar strings e reduzir alocações
/// Especialmente útil para keys repetitivas
#[derive(Debug, Clone)]
pub struct StringPool {
    pool: HashMap<String, Arc<str>>,
}

impl StringPool {
    #[inline]
    pub fn new() -> Self {
        Self {
            pool: HashMap::with_capacity(512),
        }
    }

    /// Get or intern a string
    #[inline]
    pub fn get_or_intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.pool.get(s) {
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = s.into();
            self.pool.insert(s.to_string(), Arc::clone(&interned));
            interned
        }
    }

    /// Clear the pool if it gets too large
    #[inline]
    pub fn clear_if_large(&mut self) {
        if self.pool.len() > 10_000 {
            self.pool.clear();
        }
    }

    /// Clear the entire pool
    #[inline]
    pub fn clear(&mut self) {
        self.pool.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pool.len()
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}
