//! Cache implementation without prefetch hints for performance comparison

use crate::error::Error;
use crate::event::Event;
use crate::filter::Filter;
use crate::list_props::{ListProps, Order};
use indexmap::IndexMap;
use std::fmt::{self, Debug};
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CacheItem<T> {
    pub(crate) value: T,
    pub(crate) created_at: u64,
    pub(crate) ttl_millis: Option<u64>,
}

impl<T> CacheItem<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: None,
        }
    }

    pub fn with_ttl(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: Some(ttl.as_millis() as u64),
        }
    }
}

pub struct CacheNoPrefetch<Value = String>
where
    Value: Clone + Debug + Send + Sync + 'static,
{
    map: IndexMap<String, CacheItem<Value>>,
    capacity: usize,
    event_sender: Option<Sender<Event>>,
}

impl<Value> CacheNoPrefetch<Value>
where
    Value: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            event_sender: None,
        }
    }

    pub fn with_sender(capacity: usize, sender: Sender<Event>) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            event_sender: Some(sender),
        }
    }

    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        let item = CacheItem::new(value.clone());
        let old_value = self.map.insert(key.clone(), item).map(|item| item.value);

        if self.map.len() > self.capacity {
            if let Some((evicted_key, evicted_item)) = self.map.shift_remove_index(0) {
                if let Some(ref sender) = self.event_sender {
                    let _ = sender.send(Event::remove(evicted_key, evicted_item.value.into()));
                }
            }
        }

        if let Some(ref sender) = self.event_sender {
            match old_value {
                Some(_) => {
                    let _ = sender.send(Event::insert(key, value.into()));
                }
                None => {
                    let _ = sender.send(Event::insert(key, value.into()));
                }
            }
        }

        old_value
    }

    pub fn get(&mut self, key: &str) -> Option<&Value> {
        // NO PREFETCH HERE

        let is_expired = match self.map.get(key) {
            Some(item) => {
                if let Some(ttl) = item.ttl_millis {
                    (current_time_millis() - item.created_at) > ttl
                } else {
                    false
                }
            }
            None => return None,
        };

        if is_expired {
            self.map.shift_remove(key);
            return None;
        }

        if let Some((index, _, item)) = self.map.get_full_mut(key) {
            self.map.move_index(index, self.map.len() - 1);
            Some(&item.value)
        } else {
            None
        }
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let current_time = current_time_millis();
        let mut expired_keys = Vec::with_capacity(self.map.len() / 4);

        for (key, item) in &self.map {
            // NO PREFETCH HERE

            if let Some(ttl) = item.ttl_millis {
                if (current_time - item.created_at) > ttl {
                    expired_keys.push(key.clone());
                }
            }
        }

        let removed_count = expired_keys.len();

        if !expired_keys.is_empty() {
            // NO PREFETCH HERE
            for key in expired_keys {
                if let Some(removed_item) = self.map.shift_remove(&key) {
                    if let Some(ref sender) = self.event_sender {
                        let _ = sender.send(Event::remove(key, removed_item.value.into()));
                    }
                }
            }
        }

        removed_count
    }

    pub fn list(&mut self, props: ListProps) -> Result<Vec<(String, Value)>, Error> {
        self.cleanup_expired();

        let mut keys: Vec<String> = self.map.keys().cloned().collect();
        keys.sort();

        // NO PREFETCH HERE

        match props.order {
            Order::Asc => self.resolve_order(keys.iter(), props),
            Order::Desc => self.resolve_order(keys.iter().rev(), props),
        }
    }

    fn resolve_order<'a, I>(&self, keys: I, props: ListProps) -> Result<Vec<(String, Value)>, Error>
    where
        I: Iterator<Item = &'a String>,
    {
        let mut results = Vec::new();
        let mut count = 0;

        for key in keys {
            if count >= props.limit {
                break;
            }

            if let Some(ref filter) = props.filter {
                if !self.matches_filter(key, filter) {
                    continue;
                }
            }

            if let Some(item) = self.map.get(key) {
                results.push((key.clone(), item.value.clone()));
                count += 1;
            }
        }

        Ok(results)
    }

    fn matches_filter(&self, key: &str, filter: &Filter) -> bool {
        match filter {
            Filter::StartWith(prefix) => key.starts_with(prefix),
            Filter::EndWith(suffix) => key.ends_with(suffix),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn contains_key(&self, key: &str) -> bool {
        if let Some(item) = self.map.get(key) {
            if let Some(ttl) = item.ttl_millis {
                let current_time = current_time_millis();
                (current_time - item.created_at) <= ttl
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: &str) -> Result<Option<Value>, Error> {
        if let Some(item) = self.map.shift_remove(key) {
            let value = item.value;

            if let Some(ref sender) = self.event_sender {
                let _ = sender.send(Event::remove(key.to_string(), value.clone().into()));
            }

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
