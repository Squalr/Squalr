use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeMetaData {
    None,
    SizedContainer(u64),
}
