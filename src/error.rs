use std::fmt::Debug;
use std::fmt::Display;

#[derive(PartialEq)]
pub enum Error {
    SortKeyNotFound,
    CacheAlreadyExists,
    SortKeyExists,
    TableAlreadyExists,
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
