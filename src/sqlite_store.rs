//! SQLite persistence support for Quickleaf cache.
//!
//! This module provides a simple and efficient persistence layer using SQLite.
//! Much simpler and more efficient than Parquet for cache operations.

#![cfg(feature = "persist")]

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, Result};

use crate::cache::CacheItem;
use crate::event::Event;
use crate::valu3::traits::ToValueBehavior;

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
    // Create main cache table
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
    
    // Create indices for performance
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
pub(crate) fn items_from_db(path: &Path) -> Result<Vec<(String, CacheItem)>, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    init_database(&conn)?;
    
    // Clean up expired items first
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64;
    
    conn.execute(
        "DELETE FROM cache_items WHERE expires_at IS NOT NULL AND expires_at < ?",
        params![now],
    )?;
    
    // Load all valid items
    let mut stmt = conn.prepare(
        "SELECT key, value, created_at, ttl_seconds 
         FROM cache_items 
         WHERE expires_at IS NULL OR expires_at >= ?"
    )?;
    
    let items = stmt.query_map(params![now], |row| {
        let key: String = row.get(0)?;
        let value_str: String = row.get(1)?;
        let created_at_secs: i64 = row.get(2)?;
        let ttl_seconds: Option<i64> = row.get(3)?;
        
        let value = value_str.to_value();
        let created_at = UNIX_EPOCH + Duration::from_secs(created_at_secs as u64);
        let ttl = ttl_seconds.map(|secs| Duration::from_secs(secs as u64));
        
        Ok((key, CacheItem {
            value,
            created_at,
            ttl,
        }))
    })?;
    
    let mut result = Vec::new();
    for item in items {
        result.push(item?);
    }
    
    Ok(result)
}

/// Ensure the database file exists and is initialized
pub(crate) fn ensure_db_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Open connection (creates file if doesn't exist) and init schema
    let conn = Connection::open(path)?;
    init_database(&conn)?;
    
    Ok(())
}

/// Background worker for persisting events to SQLite
pub(crate) struct SqliteWriter {
    path: PathBuf,
    receiver: Receiver<PersistentEvent>,
    conn: Connection,
}

impl SqliteWriter {
    pub fn new(path: PathBuf, receiver: Receiver<PersistentEvent>) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(&path)?;
        init_database(&conn)?;
        
        // Set pragmas for performance
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = 10000;
             PRAGMA temp_store = MEMORY;"
        )?;
        
        Ok(Self {
            path,
            receiver,
            conn,
        })
    }
    
    pub fn run(mut self) {
        loop {
            // Try to receive with timeout
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    if let Err(e) = self.process_event(&event) {
                        eprintln!("Error processing event: {}", e);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic cleanup of expired items
                    if let Err(e) = self.cleanup_expired() {
                        eprintln!("Error cleaning up expired items: {}", e);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel closed, exit
                    break;
                }
            }
        }
    }
    
    fn process_event(&mut self, event: &PersistentEvent) -> Result<()> {
        let timestamp = event.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        match &event.event {
            Event::Insert(data) => {
                let value_str = data.value.to_string();
                
                // Insert or update cache item
                // Note: We don't have TTL info in the event, so we'll handle it separately
                self.conn.execute(
                    "INSERT OR REPLACE INTO cache_items (key, value, created_at, ttl_seconds, expires_at) 
                     VALUES (?, ?, ?, NULL, NULL)",
                    params![&data.key, &value_str, timestamp],
                )?;
            }
            Event::Remove(data) => {
                self.conn.execute(
                    "DELETE FROM cache_items WHERE key = ?",
                    params![&data.key],
                )?;
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
pub(crate) fn spawn_writer(path: PathBuf, receiver: Receiver<PersistentEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        match SqliteWriter::new(path, receiver) {
            Ok(writer) => writer.run(),
            Err(e) => eprintln!("Failed to create SQLite writer: {}", e),
        }
    })
}

/// Insert with TTL support - helper function to update TTL
pub(crate) fn update_item_ttl(path: &Path, key: &str, ttl_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64;
    
    let expires_at = now + ttl_seconds as i64;
    
    conn.execute(
        "UPDATE cache_items SET ttl_seconds = ?, expires_at = ? WHERE key = ?",
        params![ttl_seconds as i64, expires_at, key],
    )?;
    
    Ok(())
}
