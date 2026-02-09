use anyhow::Result;
use common::RmpSerializable;
use common::get_env_var;
use control_server::{ControlCommand, StartCbba};
use transport::Transport;
use udp_transport::UdpTransport;

async fn send() -> Result<()> {
    let port = get_env_var("COMMAND_CONTROL_PORT");
    let mut transport = UdpTransport::new_sender(port).await?;

    let cmd: ControlCommand = StartCbba.try_into()?;
    let bytes = cmd.to_bytes()?;

    transport.send(&bytes).await?;

    Ok(())
}

pub async fn run() {
    if send().await.is_ok() {
        println!("'start cbba' command sent successfully.");
    } else {
        println!("Tasks have been sent.");
    }
}
