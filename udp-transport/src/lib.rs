use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio_util::{codec::LengthDelimitedCodec, udp::UdpFramed};
use transport::Transport;

pub use transport::TransportError;

pub struct UdpTransport {
    framed: UdpFramed<LengthDelimitedCodec>,
    broadcast_addr: Option<SocketAddr>, // Only used for sender
}

impl UdpTransport {
    pub async fn new_sender(broadcast_port: u16) -> Result<Self, TransportError> {
        let broadcast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), broadcast_port);

        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        sock.set_broadcast(true)?;
        let framed = UdpFramed::new(sock, LengthDelimitedCodec::new());

        Ok(Self {
            framed,
            broadcast_addr: Some(broadcast_addr),
        })
    }

    pub async fn new_receiver(listen_port: u16) -> Result<Self, TransportError> {
        let sock = UdpSocket::bind(("0.0.0.0", listen_port)).await?;
        let framed = UdpFramed::new(sock, LengthDelimitedCodec::new());

        Ok(Self {
            framed,
            broadcast_addr: None,
        })
    }
}

impl Transport for UdpTransport {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        let addr = self.broadcast_addr.ok_or(TransportError::NotSender)?;
        let msg = Bytes::copy_from_slice(bytes);
        self.framed.send((msg, addr)).await?;

        Ok(())
    }

    async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        match self.framed.next().await {
            Some(Ok((bytes, _addr))) => {
                if buffer.len() < bytes.len() {
                    return Err(TransportError::BufferTooSmall);
                }

                buffer[..bytes.len()].copy_from_slice(&bytes);
                Ok(bytes.len())
            }
            Some(Err(e)) => Err(e.into()),
            None => Err(TransportError::Closed),
        }
    }
}
