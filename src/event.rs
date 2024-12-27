use crate::cache::Key;

#[derive(Clone, Debug, PartialEq)]
pub enum Event<V> {
    Insert(EventData<V>),
    Remove(EventData<V>),
    Clear,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventData<V> {
    pub key: Key,
    pub value: V,
}

impl<V> Event<V> {
    pub fn insert(key: Key, value: V) -> Self {
        Self::Insert(EventData { key, value })
    }

    pub fn remove(key: Key, value: V) -> Self {
        Self::Remove(EventData { key, value })
    }

    pub fn clear() -> Self {
        Self::Clear
    }
}
