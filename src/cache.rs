use std::collections::HashMap;
use std::fmt::Debug;

use valu3::traits::ToValueBehavior;
use valu3::value::Value;

use crate::error::Error;
use crate::event::Event;
use crate::filter::Filter;
use crate::list_props::{ListProps, Order, StartAfter};
use std::sync::mpsc::Sender;

pub type Key = String;

#[derive(Clone, Debug)]
pub struct Cache {
    map: HashMap<Key, Value>,
    list: Vec<Key>,
    capacity: usize,
    sender: Option<Sender<Event<Value>>>,
    _phantom: std::marker::PhantomData<Value>,
}

impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.list == other.list && self.capacity == other.capacity
    }
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            list: Vec::new(),
            capacity,
            sender: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_sender(capacity: usize, sender: Sender<Event<Value>>) -> Self {
        Self {
            map: HashMap::new(),
            list: Vec::new(),
            capacity,
            sender: Some(sender),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set_event(&mut self, sender: Sender<Event<Value>>) {
        self.sender = Some(sender);
    }

    pub fn remove_event(&mut self) {
        self.sender = None;
    }

    fn send_insert(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::insert(key, value);
            sender.send(event).unwrap();
        }
    }

    fn send_remove(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::remove(key, value);
            sender.send(event).unwrap();
        }
    }

    fn send_clear(&self) {
        if let Some(sender) = &self.sender {
            let event = Event::clear();
            sender.send(event).unwrap();
        }
    }

    pub fn insert<T, V>(&mut self, key: T, value: V)
    where
        T: Into<String> + Clone + AsRef<str>,
        V: ToValueBehavior,
    {
        let key = key.into();

        if let Some(value) = self.map.get(&key) {
            if value.eq(&value) {
                return;
            }
        }

        if self.map.len() != 0 && self.map.len() == self.capacity {
            let first_key = self.list.remove(0);
            let data = self.map.get(&first_key).unwrap().clone();
            self.map.remove(&first_key);
            self.send_remove(first_key, data);
        }

        let position = self
            .list
            .iter()
            .position(|k| k > &key)
            .unwrap_or(self.list.len());

        self.list.insert(position, key.clone());
        self.map.insert(key.clone(), value.to_value());

        self.send_insert(key, value.to_value());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
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

                let data = self.map.get(key).unwrap().clone();

                self.map.remove(key);

                self.send_remove(key.to_string(), data);

                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();

        self.send_clear();
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

    pub fn list<T>(&self, props: T) -> Result<Vec<(Key, &Value)>, Error>
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
    ) -> Result<Vec<(Key, &Value)>, Error>
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
