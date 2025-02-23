use crate::interprocess_egress::InterprocessEgress;
use serde::{de::DeserializeOwned, Serialize};

pub trait TypedResponse<T: DeserializeOwned + Serialize>: Sized {
    fn to_response(&self) -> InterprocessEgress<T>;
    fn from_response(response: InterprocessEgress<T>) -> Result<Self, InterprocessEgress<T>>;
}
