//! Parquet persistence support for Quickleaf cache with direct Value mapping.
//!
//! This module provides functionality to persist cache operations to a Parquet file
//! with direct mapping of valu3::Value types to Parquet columns for better efficiency.

#![cfg(feature = "parquet")]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringArray, StringBuilder,
    TimestampMicrosecondBuilder, BinaryBuilder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use arrow_array::{Array, BooleanArray, Float64Array, Int64Array, TimestampMicrosecondArray, BinaryArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};

use crate::cache::CacheItem;
use crate::event::Event;
use crate::valu3::value::Value;

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

/// Parquet schema optimized for valu3::Value types
/// Instead of serializing the entire Value as binary, we use typed columns
fn create_schema() -> Schema {
    Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        
        // Value type indicator (String, Integer, Float, Boolean, etc.)
        Field::new("value_type", DataType::Utf8, false),
        
        // Typed value columns (only one will be populated per row)
        Field::new("value_string", DataType::Utf8, true),
        Field::new("value_int", DataType::Int64, true),
        Field::new("value_float", DataType::Float64, true),
        Field::new("value_bool", DataType::Boolean, true),
        Field::new("value_binary", DataType::Binary, true), // For complex types (Array, Object, etc.)
        
        // Metadata columns
        Field::new(
            "created_at",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
        Field::new("ttl_seconds", DataType::Int64, true),
        Field::new("operation_type", DataType::Utf8, false),
        Field::new(
            "operation_timestamp",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
    ])
}

/// Convert a persistent event to a RecordBatch with direct Value mapping
fn event_to_record_batch(event: &PersistentEvent) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let schema = Arc::new(create_schema());
    
    let mut key_builder = StringBuilder::new();
    let mut value_type_builder = StringBuilder::new();
    let mut value_string_builder = StringBuilder::new();
    let mut value_int_builder = Int64Builder::new();
    let mut value_float_builder = Float64Builder::new();
    let mut value_bool_builder = BooleanBuilder::new();
    let mut value_binary_builder = BinaryBuilder::new();
    let mut created_at_builder = TimestampMicrosecondBuilder::new();
    let mut ttl_builder = Int64Builder::new();
    let mut operation_builder = StringBuilder::new();
    let mut op_timestamp_builder = TimestampMicrosecondBuilder::new();

    let op_timestamp = event
        .timestamp
        .duration_since(UNIX_EPOCH)?
        .as_micros() as i64;

    match &event.event {
        Event::Insert(data) => {
            key_builder.append_value(&data.key);
            
            // Map Value types directly to Parquet columns
            match &data.value {
                Value::String(s) => {
                    value_type_builder.append_value("String");
                    value_string_builder.append_value(&s.value);
                    value_int_builder.append_null();
                    value_float_builder.append_null();
                    value_bool_builder.append_null();
                    value_binary_builder.append_null();
                }
                Value::Integer(i) => {
                    value_type_builder.append_value("Integer");
                    value_string_builder.append_null();
                    value_int_builder.append_value(i.value);
                    value_float_builder.append_null();
                    value_bool_builder.append_null();
                    value_binary_builder.append_null();
                }
                Value::Float(f) => {
                    value_type_builder.append_value("Float");
                    value_string_builder.append_null();
                    value_int_builder.append_null();
                    value_float_builder.append_value(f.value);
                    value_bool_builder.append_null();
                    value_binary_builder.append_null();
                }
                Value::Boolean(b) => {
                    value_type_builder.append_value("Boolean");
                    value_string_builder.append_null();
                    value_int_builder.append_null();
                    value_float_builder.append_null();
                    value_bool_builder.append_value(b.value);
                    value_binary_builder.append_null();
                }
                // For complex types (Array, Object, etc.), fall back to bincode
                _ => {
                    value_type_builder.append_value("Complex");
                    value_string_builder.append_null();
                    value_int_builder.append_null();
                    value_float_builder.append_null();
                    value_bool_builder.append_null();
                    let bytes = bincode::serialize(&data.value)?;
                    value_binary_builder.append_value(&bytes);
                }
            }
            
            let created_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_micros() as i64;
            created_at_builder.append_value(created_at);
            ttl_builder.append_null();
            operation_builder.append_value("INSERT");
            op_timestamp_builder.append_value(op_timestamp);
        }
        Event::Remove(data) => {
            key_builder.append_value(&data.key);
            
            // For remove, we still need to record the value type
            value_type_builder.append_value("Remove");
            value_string_builder.append_null();
            value_int_builder.append_null();
            value_float_builder.append_null();
            value_bool_builder.append_null();
            value_binary_builder.append_null();
            
            created_at_builder.append_value(op_timestamp);
            ttl_builder.append_null();
            operation_builder.append_value("REMOVE");
            op_timestamp_builder.append_value(op_timestamp);
        }
        Event::Clear => {
            key_builder.append_value("");
            value_type_builder.append_value("Clear");
            value_string_builder.append_null();
            value_int_builder.append_null();
            value_float_builder.append_null();
            value_bool_builder.append_null();
            value_binary_builder.append_null();
            
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
            Arc::new(value_type_builder.finish()) as ArrayRef,
            Arc::new(value_string_builder.finish()) as ArrayRef,
            Arc::new(value_int_builder.finish()) as ArrayRef,
            Arc::new(value_float_builder.finish()) as ArrayRef,
            Arc::new(value_bool_builder.finish()) as ArrayRef,
            Arc::new(value_binary_builder.finish()) as ArrayRef,
            Arc::new(created_at_builder.finish()) as ArrayRef,
            Arc::new(ttl_builder.finish()) as ArrayRef,
            Arc::new(operation_builder.finish()) as ArrayRef,
            Arc::new(op_timestamp_builder.finish()) as ArrayRef,
        ],
    )?;

    Ok(batch)
}

/// Read cache items from a Parquet file with direct Value reconstruction
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
        
        let keys = batch.column(0).as_any().downcast_ref::<StringArray>()
            .ok_or("Failed to cast key column")?;
        let value_types = batch.column(1).as_any().downcast_ref::<StringArray>()
            .ok_or("Failed to cast value_type column")?;
        let value_strings = batch.column(2).as_any().downcast_ref::<StringArray>()
            .ok_or("Failed to cast value_string column")?;
        let value_ints = batch.column(3).as_any().downcast_ref::<Int64Array>()
            .ok_or("Failed to cast value_int column")?;
        let value_floats = batch.column(4).as_any().downcast_ref::<Float64Array>()
            .ok_or("Failed to cast value_float column")?;
        let value_bools = batch.column(5).as_any().downcast_ref::<BooleanArray>()
            .ok_or("Failed to cast value_bool column")?;
        let value_binaries = batch.column(6).as_any().downcast_ref::<BinaryArray>()
            .ok_or("Failed to cast value_binary column")?;
        let created_ats = batch.column(7).as_any().downcast_ref::<TimestampMicrosecondArray>()
            .ok_or("Failed to cast created_at column")?;
        let ttls = batch.column(8).as_any().downcast_ref::<Int64Array>()
            .ok_or("Failed to cast ttl column")?;
        let operations = batch.column(9).as_any().downcast_ref::<StringArray>()
            .ok_or("Failed to cast operation column")?;
        let op_timestamps = batch.column(10).as_any().downcast_ref::<TimestampMicrosecondArray>()
            .ok_or("Failed to cast operation_timestamp column")?;

        for i in 0..batch.num_rows() {
            let operation = operations.value(i);
            let op_timestamp = op_timestamps.value(i);
            
            match operation {
                "INSERT" => {
                    let key = keys.value(i).to_string();
                    let value_type = value_types.value(i);
                    
                    // Reconstruct Value from typed columns
                    let value = match value_type {
                        "String" => {
                            if value_strings.is_valid(i) {
                                value_strings.value(i).to_value()
                            } else {
                                continue;
                            }
                        }
                        "Integer" => {
                            if value_ints.is_valid(i) {
                                value_ints.value(i).to_value()
                            } else {
                                continue;
                            }
                        }
                        "Float" => {
                            if value_floats.is_valid(i) {
                                value_floats.value(i).to_value()
                            } else {
                                continue;
                            }
                        }
                        "Boolean" => {
                            if value_bools.is_valid(i) {
                                value_bools.value(i).to_value()
                            } else {
                                continue;
                            }
                        }
                        "Complex" => {
                            if value_binaries.is_valid(i) {
                                let bytes = value_binaries.value(i);
                                bincode::deserialize(bytes)?
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    };
                    
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
                    
                    match items.get(&key) {
                        Some((_, existing_timestamp)) if *existing_timestamp > op_timestamp => {
                            // Keep existing, more recent operation
                        }
                        _ => {
                            items.insert(key, (item, op_timestamp));
                        }
                    }
                }
                "REMOVE" => {
                    let key = keys.value(i).to_string();
                    match items.get(&key) {
                        Some((_, existing_timestamp)) if *existing_timestamp > op_timestamp => {
                            // Keep existing, more recent operation
                        }
                        _ => {
                            items.remove(&key);
                        }
                    }
                }
                "CLEAR" => {
                    if clear_timestamp.is_none() || clear_timestamp.unwrap() < op_timestamp {
                        clear_timestamp = Some(op_timestamp);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(clear_ts) = clear_timestamp {
        items.retain(|_, (_, timestamp)| *timestamp > clear_ts);
    }

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

// Rest of the code remains the same...
