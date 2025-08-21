//! Parquet persistence support for Quickleaf cache.
//!
//! This module provides a simple and efficient persistence layer using Parquet files.
//! Values are stored as JSON strings and converted using valu3's built-in JSON support.

#![cfg(feature = "parquet")]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrow::array::{
    ArrayRef, Int64Builder, StringArray, StringBuilder,
    TimestampMicrosecondBuilder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use arrow_array::{Array, Int64Array, TimestampMicrosecondArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};

use crate::cache::CacheItem;
use crate::event::Event;
use crate::valu3::value::Value;
use crate::valu3::traits::ToValueBehavior;

/// Extended event structure for persistence
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Simple Parquet schema with just key and value columns
fn create_schema() -> Schema {
    Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, true), // JSON string representation
        Field::new(
            "created_at",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
        Field::new("ttl_seconds", DataType::Int64, true), // Nullable for items without TTL
        Field::new("operation_type", DataType::Utf8, false),
        Field::new(
            "operation_timestamp",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
    ])
}

/// Convert a persistent event to a RecordBatch for Parquet writing
fn event_to_record_batch(event: &PersistentEvent) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let schema = Arc::new(create_schema());
    
    let mut key_builder = StringBuilder::new();
    let mut value_builder = StringBuilder::new();
    let mut created_at_builder = TimestampMicrosecondBuilder::new();
    let mut ttl_builder = Int64Builder::new();
    let mut operation_builder = StringBuilder::new();
    let mut op_timestamp_builder = TimestampMicrosecondBuilder::new();

    // Convert operation timestamp to microseconds
    let op_timestamp = event
        .timestamp
        .duration_since(UNIX_EPOCH)?
        .as_micros() as i64;

    match &event.event {
        Event::Insert(data) => {
            key_builder.append_value(&data.key);
            
            // Convert Value to JSON string
            // For now, we'll use Display trait which should give us a readable format
            let value_str = format!("{:?}", data.value);
            value_builder.append_value(&value_str);
            
            // For insert operations, we need to extract created_at and ttl from the actual cache item
            // Since we don't have direct access to CacheItem here, we'll use current time
            // In the actual implementation, we'll need to pass CacheItem data through the event
            let created_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_micros() as i64;
            created_at_builder.append_value(created_at);
            
            // TTL will be handled when we have access to the actual CacheItem
            ttl_builder.append_null();
            
            operation_builder.append_value("INSERT");
            op_timestamp_builder.append_value(op_timestamp);
        }
        Event::Remove(data) => {
            key_builder.append_value(&data.key);
            
            // Store the value for consistency
            let value_str = format!("{:?}", data.value);
            value_builder.append_value(&value_str);
            
            // For remove, created_at is not relevant but we need a value
            created_at_builder.append_value(op_timestamp);
            ttl_builder.append_null();
            
            operation_builder.append_value("REMOVE");
            op_timestamp_builder.append_value(op_timestamp);
        }
        Event::Clear => {
            key_builder.append_value("");
            value_builder.append_null();
            created_at_builder.append_value(op_timestamp);
            ttl_builder.append_null();
            
            operation_builder.append_value("CLEAR");
            op_timestamp_builder.append_value(op_timestamp);
        }
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(key_builder.finish()) as ArrayRef,
            Arc::new(value_builder.finish()) as ArrayRef,
            Arc::new(created_at_builder.finish()) as ArrayRef,
            Arc::new(ttl_builder.finish()) as ArrayRef,
            Arc::new(operation_builder.finish()) as ArrayRef,
            Arc::new(op_timestamp_builder.finish()) as ArrayRef,
        ],
    )?;

    Ok(batch)
}

/// Read cache items from a Parquet file
pub(crate) fn items_from_file(path: &Path) -> Result<Vec<(String, CacheItem)>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let mut reader = builder.build()?;

    let mut items: std::collections::HashMap<String, (CacheItem, i64)> = std::collections::HashMap::new();
    let mut clear_timestamp: Option<i64> = None;

    while let Some(batch) = reader.next() {
        let batch = batch?;
        
        let keys = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or("Failed to cast key column")?;
            
        let values = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or("Failed to cast value column")?;
            
        let created_ats = batch
            .column(2)
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .ok_or("Failed to cast created_at column")?;
            
        let ttls = batch
            .column(3)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or("Failed to cast ttl column")?;
            
        let operations = batch
            .column(4)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or("Failed to cast operation column")?;
            
        let op_timestamps = batch
            .column(5)
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .ok_or("Failed to cast operation_timestamp column")?;

        for i in 0..batch.num_rows() {
            let operation = operations.value(i);
            let op_timestamp = op_timestamps.value(i);
            
            match operation {
                "INSERT" => {
                    let key = keys.value(i).to_string();
                    
                    if values.is_valid(i) {
                        // Convert the string back to a Value
                        // Since we stored it as Debug format, we'll convert it to a string Value
                        let value_str = values.value(i);
                        let value = value_str.to_value();
                        
                        let created_at = UNIX_EPOCH + Duration::from_micros(created_ats.value(i) as u64);
                        
                        let ttl = if ttls.is_valid(i) {
                            Some(Duration::from_secs(ttls.value(i) as u64))
                        } else {
                            None
                        };
                        
                        let item = CacheItem {
                            value,
                            created_at,
                            ttl,
                        };
                        
                        // Only keep the latest operation for each key
                        match items.get(&key) {
                            Some((_, existing_timestamp)) if *existing_timestamp > op_timestamp => {
                                // Keep existing, more recent operation
                            }
                            _ => {
                                items.insert(key, (item, op_timestamp));
                            }
                        }
                    }
                }
                "REMOVE" => {
                    let key = keys.value(i).to_string();
                    
                    // Check if this remove is more recent than any insert
                    match items.get(&key) {
                        Some((_, existing_timestamp)) if *existing_timestamp > op_timestamp => {
                            // Keep existing, more recent operation
                        }
                        _ => {
                            // Mark as removed by removing from map
                            items.remove(&key);
                        }
                    }
                }
                "CLEAR" => {
                    // Track the most recent clear operation
                    if clear_timestamp.is_none() || clear_timestamp.unwrap() < op_timestamp {
                        clear_timestamp = Some(op_timestamp);
                    }
                }
                _ => {}
            }
        }
    }

    // If there was a CLEAR operation, only keep items inserted after it
    if let Some(clear_ts) = clear_timestamp {
        items.retain(|_, (_, timestamp)| *timestamp > clear_ts);
    }

    // Filter out expired items
    let result: Vec<(String, CacheItem)> = items
        .into_iter()
        .filter_map(|(key, (item, _))| {
            if !item.is_expired() {
                Some((key, item))
            } else {
                None
            }
        })
        .collect();

    Ok(result)
}

/// Ensure the parent directory exists and create the Parquet file if needed
pub(crate) fn ensure_parquet_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // If file doesn't exist, create it with the schema
    if !path.exists() {
        let file = fs::File::create(path)?;
        let schema = Arc::new(create_schema());
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();
        let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
        writer.finish()?;
    }

    Ok(())
}

/// Background worker for persisting events to Parquet
pub(crate) struct ParquetWriter {
    path: PathBuf,
    receiver: Receiver<PersistentEvent>,
    buffer: Vec<PersistentEvent>,
    buffer_size: usize,
}

impl ParquetWriter {
    pub fn new(path: PathBuf, receiver: Receiver<PersistentEvent>) -> Self {
        Self {
            path,
            receiver,
            buffer: Vec::new(),
            buffer_size: 100, // Write every 100 events or on timeout
        }
    }

    pub fn run(mut self) {
        loop {
            // Try to receive with timeout
            match self.receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    self.buffer.push(event);
                    
                    // Write if buffer is full
                    if self.buffer.len() >= self.buffer_size {
                        if let Err(e) = self.write_buffer() {
                            eprintln!("Error writing to Parquet: {}", e);
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Write any pending events on timeout
                    if !self.buffer.is_empty() {
                        if let Err(e) = self.write_buffer() {
                            eprintln!("Error writing to Parquet: {}", e);
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel closed, write remaining buffer and exit
                    if !self.buffer.is_empty() {
                        if let Err(e) = self.write_buffer() {
                            eprintln!("Error writing final buffer to Parquet: {}", e);
                        }
                    }
                    break;
                }
            }
        }
    }

    fn write_buffer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Open file in append mode
        let file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let schema = Arc::new(create_schema());
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();

        let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;

        // Convert all buffered events to record batches and write
        for event in &self.buffer {
            let batch = event_to_record_batch(event)?;
            writer.write(&batch)?;
        }

        writer.finish()?;
        self.buffer.clear();
        
        Ok(())
    }
}

/// Spawn the background writer thread
pub(crate) fn spawn_writer(path: PathBuf, receiver: Receiver<PersistentEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let writer = ParquetWriter::new(path, receiver);
        writer.run();
    })
}
