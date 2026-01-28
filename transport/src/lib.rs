#![allow(async_fn_in_trait)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("buffer too small")]
    BufferTooSmall,

    #[error("transport is closed")]
    Closed,

    #[error("transport is not configured as sender")]
    NotSender,
}

pub trait Transport {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError>;
    async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError>;
}
