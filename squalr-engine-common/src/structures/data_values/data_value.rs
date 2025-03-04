use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

pub trait DataValue: Debug + Send + Sync {
    fn get_size_in_bytes(&self) -> u64;
    fn get_value_string(&self) -> String;
    fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    );

    fn clone_internal(&self) -> Box<dyn DataValue>;
    fn serialize_internal(&self) -> Value;
}

impl Clone for Box<dyn DataValue> {
    fn clone(&self) -> Box<dyn DataValue> {
        self.clone_internal()
    }
}

impl Serialize for Box<dyn DataValue> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("value", &self.get_value_string())?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Box<dyn DataValue> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        panic!("TODO")
    }
}

impl FromStr for Box<dyn DataValue> {
    type Err = serde_json::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        panic!("TODO")
    }
}

impl fmt::Display for dyn DataValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_value_string())
    }
}
