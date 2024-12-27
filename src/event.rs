use valu3::value::Value;

use crate::cache::Key;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Insert(EventData),
    Remove(EventData),
    Clear,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventData {
    pub key: Key,
    pub value: Value,
}

impl Event {
    pub fn insert(key: Key, value: Value) -> Self {
        Self::Insert(EventData { key, value })
    }

    pub fn remove(key: Key, value: Value) -> Self {
        Self::Remove(EventData { key, value })
    }

    pub fn clear() -> Self {
        Self::Clear
    }
}
