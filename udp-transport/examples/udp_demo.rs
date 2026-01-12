use tokio::time::{Duration, sleep};
use transport::Transport;
use udp_transport::UdpTransport;

// Run as cargo run --example udp_demo --package udp-transport

#[tokio::main]
async fn main() -> Result<(), transport::TransportError> {
    // Create a sender (broadcast port 4000)
    let mut sender = UdpTransport::new_sender(4000).await?;

    // Create a receiver (listen on port 4000)
    let mut receiver = UdpTransport::new_receiver(4000).await?;

    // Spawn a task to receive messages
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        loop {
            if let Ok(Some(len)) = receiver.recv(&mut buf).await {
                println!("Received: {:?}", &buf[..len]);
            }
        }
    });

    // Send messages every second
    for i in 0..5 {
        let msg = format!("Hello {}", i);
        sender.send(msg.as_bytes()).await?;
        println!("Sent: {}", msg);
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
