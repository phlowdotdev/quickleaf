//! SQLite persistence support for Quickleaf cache.
//!
//! This module provides a simple and efficient persistence layer using SQLite
//! for durable cache storage.

#![cfg(feature = "persist")]

use crate::cache::CacheItem;
use crate::event::Event;
use crate::valu3::prelude::*;
use crate::valu3::traits::ToValueBehavior;
use rusqlite::{params, Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Extended event structure for persistence
#[derive(Clone, Debug)]
pub(crate) struct PersistentEvent {
    pub event: Event,
    pub timestamp: SystemTime,
}

impl PersistentEvent {
    pub fn new(event: Event) -> Self {
        Self {
            event,
            timestamp: SystemTime::now(),
        }
    }
}

/// Initialize SQLite database with schema
fn init_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cache_items (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            ttl_seconds INTEGER,
            expires_at INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expires 
         ON cache_items(expires_at) 
         WHERE expires_at IS NOT NULL",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_created 
         ON cache_items(created_at)",
        [],
    )?;

    Ok(())
}

/// Read cache items from SQLite database
pub(crate) fn items_from_db(
    path: &Path,
) -> Result<Vec<(String, CacheItem)>, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    init_database(&conn)?;

    let _ = conn.execute_batch("PRAGMA journal_mode = DELETE;");
    let _ = conn.execute_batch("PRAGMA busy_timeout = 5000;");

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    conn.execute(
        "DELETE FROM cache_items WHERE expires_at IS NOT NULL AND expires_at < ?",
        params![now],
    )?;

    let mut stmt = conn.prepare(
        "SELECT key, value, created_at, ttl_seconds 
         FROM cache_items 
         WHERE expires_at IS NULL OR expires_at >= ?",
    )?;

    let items = stmt.query_map(params![now], |row| {
        let key: String = row.get(0)?;
        let value_json: String = row.get(1)?;
        let created_at_secs: i64 = row.get(2)?;
        let ttl_seconds: Option<i64> = row.get(3)?;

        let value = Value::json_to_value(&value_json).unwrap_or_else(|_| value_json.to_value());
        let created_at = created_at_secs as u64 * 1000;
        let ttl_millis = ttl_seconds.map(|secs| secs as u64 * 1000);

        Ok((
            key,
            CacheItem {
                value,
                created_at,
                ttl_millis,
            },
        ))
    })?;

    let mut result = Vec::new();
    for item in items {
        result.push(item?);
    }

    Ok(result)
}

/// Ensure the database file exists and is initialized
pub(crate) fn ensure_db_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(path)?;
    init_database(&conn)?;

    let _ = conn.execute_batch("PRAGMA journal_mode = DELETE;");
    let _ = conn.execute_batch("PRAGMA busy_timeout = 5000;");

    Ok(())
}

/// Background worker for persisting events to SQLite
pub(crate) struct SqliteWriter {
    receiver: Receiver<PersistentEvent>,
    conn: Connection,
}

impl SqliteWriter {
    pub fn new(
        path: PathBuf,
        receiver: Receiver<PersistentEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(&path)?;
        init_database(&conn)?;

        match conn.execute_batch("PRAGMA journal_mode = WAL;") {
            Ok(_) => {}
            Err(_) => {
                let _ = conn.execute_batch("PRAGMA journal_mode = DELETE;");
            }
        }

        let _ = conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = 10000;
             PRAGMA temp_store = MEMORY;
             PRAGMA busy_timeout = 5000;",
        );

        Ok(Self { receiver, conn })
    }

    pub fn run(mut self) {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    if let Err(e) = self.process_event(&event) {
                        eprintln!("Error processing event: {}", e);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Err(e) = self.cleanup_expired() {
                        eprintln!("Error cleaning up expired items: {}", e);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    }

    fn process_event(&mut self, event: &PersistentEvent) -> Result<()> {
        let timestamp = event
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        match &event.event {
            Event::Insert(data) => {
                let value_json = data.value.to_json(JsonMode::Inline);

                self.conn.execute(
                    "INSERT OR REPLACE INTO cache_items (key, value, created_at, ttl_seconds, expires_at) 
                     VALUES (?, ?, ?, NULL, NULL)",
                    params![&data.key, &value_json, timestamp],
                )?;
            }
            Event::Remove(data) => {
                self.conn
                    .execute("DELETE FROM cache_items WHERE key = ?", params![&data.key])?;
            }
            Event::Clear => {
                self.conn.execute("DELETE FROM cache_items", [])?;
            }
        }

        Ok(())
    }

    fn cleanup_expired(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "DELETE FROM cache_items WHERE expires_at IS NOT NULL AND expires_at < ?",
            params![now],
        )?;

        Ok(())
    }
}

/// Spawn the background writer thread
pub(crate) fn spawn_writer(
    path: PathBuf,
    receiver: Receiver<PersistentEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || match SqliteWriter::new(path, receiver) {
        Ok(writer) => writer.run(),
        Err(e) => eprintln!("Failed to create SQLite writer: {}", e),
    })
}

/// Persist an item with TTL directly to the database
pub(crate) fn persist_item_with_ttl(
    path: &Path,
    key: &str,
    value: &Value,
    ttl_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let expires_at = now + ttl_seconds as i64;
    let value_json = value.to_json(JsonMode::Inline);

    conn.execute(
        "INSERT OR REPLACE INTO cache_items (key, value, created_at, ttl_seconds, expires_at) 
         VALUES (?, ?, ?, ?, ?)",
        params![key, value_json, now, ttl_seconds as i64, expires_at],
    )?;

    Ok(())
}
