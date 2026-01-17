use super::ByteSerializable;
use crate::SerializationError;
use bytes::Bytes;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

/// Implement ByteSerializable for any type that supports `rmp_serde` serialization
impl<T> ByteSerializable for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn to_bytes(&self) -> Result<Bytes, SerializationError> {
        let mut vector = Vec::new();
        self.serialize(&mut Serializer::new(&mut vector))
            .map_err(|_| SerializationError::GenericError)?;
        Ok(Bytes::from(vector))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        rmp_serde::decode::from_slice(bytes).map_err(|_| SerializationError::GenericError)
    }
}
