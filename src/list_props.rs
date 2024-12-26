use crate::filter::Filter;

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Self::Asc
    }
}

#[derive(Debug, Clone)]
pub enum StartAfter {
    Key(&'static str),
    None,
}

impl Default for StartAfter {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Debug)]
pub struct ListProps {
    pub start_after_key: StartAfter,
    pub filter: Filter<'static>,
    pub order: Order,
    pub limit: usize,
}

impl ListProps {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }

    pub fn start_after_key(mut self, key: &'static str) -> Self {
        self.start_after_key = StartAfter::Key(key);
        self
    }

    pub fn filter(mut self, filter: Filter<'static>) -> Self {
        self.filter = filter;
        self
    }

    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl From<Filter<'static>> for ListProps {
    fn from(filter: Filter<'static>) -> Self {
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
