use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("MessagePack encode error")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("MessagePack decode error")]
    Decode(#[from] rmp_serde::decode::Error),
}

/// Trait for types that can be serialized/deserialized via MessagePack.
pub trait RmpSerializable: Serialize + for<'de> Deserialize<'de> {
    fn to_bytes(&self) -> Result<Bytes, SerializationError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

pub mod rmp;
