
# Multi-Agent Transport Demo

This demo runs multiple agents that communicate over UDP using 
the `transport` and `udp-transport` crates from the workspace.
Each agent runs in its own Docker container and exchanges messages 
on a shared network.

## Prerequisites

* Docker
* Docker Compose
* Bash (for `launch.sh`)

## Build and Run

Build the Docker image:

```bash
docker compose build
```

Start the agents:

```bash
./launch.sh
```

By default, the script starts multiple agents (containers), each with its own `AGENT_ID`.

## Stop and Cleanup

Stop all agents and tear down the setup:

```bash
docker compose down --remove-orphans
```

## Notes

* Each agent runs the same binary but is configured via environment variables.
* Networking is handled via Docker Compose bridge networking.
* This demo is intended as a foundation for discovery and CBBA experiments.
