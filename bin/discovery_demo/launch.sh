#!/bin/bash
set -euo pipefail

NUM_AGENTS=5
SERVICE_NAME=agent

if [ $# -gt 0 ]; then
    NUM_AGENTS=$1
fi

echo "Number of agents: $NUM_AGENTS"

BASE_ID=10000

for i in $(seq 0 $((NUM_AGENTS - 1))); do
    AGENT_ID=$((BASE_ID + i))
    CONTAINER_NAME="agent-$AGENT_ID"

    echo "Starting $CONTAINER_NAME with AGENT_ID=$AGENT_ID"
    docker compose run -d \
        --name $CONTAINER_NAME \
        -e AGENT_ID=$AGENT_ID \
        $SERVICE_NAME
done
