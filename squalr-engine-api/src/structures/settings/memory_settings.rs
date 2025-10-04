use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct MemorySettings {
    pub memory_type_none: bool,
    pub memory_type_private: bool,
    pub memory_type_image: bool,
    pub memory_type_mapped: bool,
    pub required_write: bool,
    pub required_execute: bool,
    pub required_copy_on_write: bool,
    pub excluded_write: bool,
    pub excluded_execute: bool,
    pub excluded_copy_on_write: bool,
    pub start_address: u64,
    pub end_address: u64,
    pub only_query_usermode: bool,
}

impl fmt::Debug for MemorySettings {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for memory: {}", json),
            Err(_) => write!(formatter, "Memory config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            memory_type_none: true,
            memory_type_private: true,
            memory_type_image: true,
            memory_type_mapped: true,

            required_write: false,
            required_execute: false,
            required_copy_on_write: false,

            excluded_write: false,
            excluded_execute: false,
            excluded_copy_on_write: false,

            start_address: 0,
            end_address: u64::MAX,
            only_query_usermode: false,
        }
    }
}
