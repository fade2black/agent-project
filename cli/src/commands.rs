use clap::{Arg, Command};

pub fn build_command() -> Command {
    Command::new("agent")
        .version("1.0")
        .author("Bayram <bkuliyev@gmail.com>")
        .about("Agent client.")
        .subcommand(Command::new("start-cbba").about("Start the CBBA process."))
        .subcommand(
            Command::new("send-cbba-gossip")
                .about("Send a CBBA gossip on behalf of an agent.")
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .value_name("FILE")
                        .help("Path to the gossip file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("distribute-tasks")
                .about("Distribute tasks to agents")
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .value_name("FILE")
                        .help("Path to the task distribution file")
                        .required(true),
                ),
        )
}
