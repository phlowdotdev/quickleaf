use std::collections::HashMap;
use std::fmt::Debug;

use crate::error::Error;
use crate::filter::Filter;
use crate::list_props::{ListProps, Order, StartAfter};

pub type Key = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Quickleaf<V>
where
    V: PartialEq,
{
    map: HashMap<Key, V>,
    list: Vec<Key>,
    capacity: usize,
    _phantom: std::marker::PhantomData<V>,
}

impl<V> Quickleaf<V>
where
    V: PartialEq,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            list: Vec::new(),
            capacity,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn insert_str(&mut self, key: &'static str, value: V) {
        self.insert(key.to_string(), value);
    }

    pub fn insert(&mut self, key: Key, value: V) {
        if let Some(value) = self.map.get(&key) {
            if value.eq(value) {
                return;
            }
        }

        if self.map.len() != 0 && self.map.len() == self.capacity {
            let first_key = self.list.remove(0);
            self.map.remove(&first_key);
        }

        // sorted by key
        let position = self
            .list
            .iter()
            .position(|k| k > &key)
            .unwrap_or(self.list.len());
        self.list.insert(position, key.to_string());
        self.map.insert(key, value.into());
    }

    pub fn insert_if_not_exists(&mut self, key: Key, value: V) -> Result<(), Error> {
        if self.map.contains_key(&key) {
            return Err(Error::SortKeyExists);
        }

        self.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        match self.list.iter().position(|k| k == &key) {
            Some(position) => {
                self.list.remove(position);
                self.map.remove(key);
                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn list<T>(&self, props: T) -> Result<Vec<(Key, &V)>, Error>
    where
        T: Into<ListProps>,
    {
        let props = props.into();

        match props.order {
            Order::Asc => self.resolve_order(self.list.iter(), props),
            Order::Desc => self.resolve_order(self.list.iter().rev(), props),
        }
    }

    fn resolve_order<'a, I>(
        &self,
        mut list_iter: I,
        props: ListProps,
    ) -> Result<Vec<(Key, &V)>, Error>
    where
        I: Iterator<Item = &'a Key>,
    {
        if let StartAfter::Key(key) = props.start_after_key {
            list_iter
                .find(|k| k == &key)
                .ok_or(Error::SortKeyNotFound)?;
        }

        let mut list = Vec::new();
        let mut count = 0;

        for k in list_iter {
            let filtered = match props.filter {
                Filter::StartWith(key) => {
                    if k.starts_with(&key) {
                        Some((k.clone(), self.map.get(k).unwrap()))
                    } else {
                        None
                    }
                }
                Filter::EndWith(key) => {
                    if k.ends_with(&key) {
                        Some((k.clone(), self.map.get(k).unwrap()))
                    } else {
                        None
                    }
                }
                Filter::StartAndEndWith(start_key, end_key) => {
                    if k.starts_with(&start_key) && k.ends_with(&end_key) {
                        Some((k.clone(), self.map.get(k).unwrap()))
                    } else {
                        None
                    }
                }
                Filter::None => Some((k.clone(), self.map.get(k).unwrap())),
            };

            if let Some(item) = filtered {
                list.push(item);
                count += 1;
                if count == props.limit {
                    break;
                }
            }
        }

        Ok(list)
    }
}
