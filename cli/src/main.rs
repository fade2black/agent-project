mod commands;
mod distribute_tasks;
mod send_gossip;
mod start_cbba;

use crate::commands::build_command;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = commands::build_command().get_matches();

    if let Some((cmd_name, matches)) = matches.subcommand() {
        match cmd_name {
            "dist" => {
                distribute_tasks::run(&matches).await?;
            }
            "cbba" => {
                start_cbba::run().await;
            }
            "send-cbba-gossip" => {
                send_gossip::run();
            }
            _ => build_command().print_help()?,
        }
    } else {
        build_command().print_help()?;
    }

    Ok(())
}
