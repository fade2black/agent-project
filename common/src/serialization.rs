use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("Deserialization Error")]
    DeserializationError(#[from] serde::de::value::Error),
    #[error("Serialization Error")]
    GenericError,
}

pub trait ByteSerializable: Serialize + for<'de> Deserialize<'de> {
    /// Convert the object to bytes (serialization).
    fn to_bytes(&self) -> Result<Bytes, SerializationError>;

    /// Create the object from bytes (deserialization).
    fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

pub mod rmp;
