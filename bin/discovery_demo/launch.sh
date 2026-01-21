#!/bin/bash
set -euo pipefail

NUM_AGENTS=5
SERVICE_NAME=agent
CONTAINER_PORT=8080  # fixed container port
BASE_HOST_PORT=8080  # first host port


if [ $# -gt 0 ]; then
    NUM_AGENTS=$1
fi

echo "Number of agents: $NUM_AGENTS"

BASE_ID=10000

for i in $(seq 0 $((NUM_AGENTS - 1))); do
    AGENT_ID=$((BASE_ID + i))
    HOST_PORT=$((BASE_HOST_PORT + i))
    CONTAINER_NAME="agent-$AGENT_ID"

    echo "Starting $CONTAINER_NAME with AGENT_ID=$AGENT_ID on host port $HOST_PORT"
    docker compose run -d \
        --name $CONTAINER_NAME \
        -e AGENT_ID=$AGENT_ID \
        -p ${HOST_PORT}:${CONTAINER_PORT} \
        $SERVICE_NAME
done
