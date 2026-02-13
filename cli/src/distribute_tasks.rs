use agent_state::{Location, Task};
use anyhow::Result;
use clap::ArgMatches;
use common::RmpSerializable;
use common::{get_env_var, time::now};
use control_server::{ControlCommand, DistributeTasks};
use serde::Deserialize;
use transport::Transport;
use udp_transport::UdpTransport;

#[derive(Debug, Deserialize)]
struct CliLocation {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct CliTask {
    id: u32,
    priority: u16,
    location: CliLocation,
}

#[derive(Debug, Deserialize)]
struct TasksFile {
    tasks: Vec<CliTask>,
}

impl From<CliTask> for Task {
    fn from(task: CliTask) -> Self {
        Task {
            id: task.id,
            priority: task.priority,
            location: Location::new(task.location.lat, task.location.lon),
            ts: now(),
        }
    }
}

fn parse_args(matches: &ArgMatches) -> Result<String> {
    let file_name = matches
        .get_one::<String>("file")
        .ok_or(anyhow::Error::msg("file argument is required"))?;

    Ok(file_name.to_string())
}

async fn send(tasks: Vec<Task>) -> Result<()> {
    let port = get_env_var("COMMAND_CONTROL_PORT");
    let mut transport = UdpTransport::new_sender(port).await?;

    let cmd: ControlCommand = DistributeTasks::new(tasks).try_into()?;
    let bytes = cmd.to_bytes()?;

    transport.send(&bytes).await?;

    Ok(())
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    let file_name = parse_args(matches)?;
    let content = std::fs::read_to_string(file_name)?;
    let tasks_file: TasksFile = serde_yaml::from_str(&content)?;

    let tasks = tasks_file.tasks.into_iter().map(Task::from).collect();

    if send(tasks).await.is_ok() {
        println!("Command sent successfully.");
    } else {
        println!("Failed to send the command.");
    }

    Ok(())
}
