#!/bin/bash
set -euo pipefail

NUM_AGENTS=5
BASE_ID=10000
SERVICE_NAME=agent
CONTAINER_PORT=8080  # fixed container port
BASE_HOST_PORT=8080  # first host port
CLI_CONTAINER_NAME="agent-cli"
# Location related constants
CENTER_LAT=52.544746455
CENTER_LON=13.439647316
DISTANCE=15 # meters
LAT_METERS=111111

if [ $# -gt 0 ]; then
    NUM_AGENTS=$1
fi

echo "Number of agents: $NUM_AGENTS"

for i in $(seq 0 $((NUM_AGENTS - 1))); do
    AGENT_ID=$((BASE_ID + i))
    HOST_PORT=$((BASE_HOST_PORT + i))
    CONTAINER_NAME="agent-$AGENT_ID"

    # Compute location
    angle=$(echo "2 * 3.14159265359 * $i / $NUM_AGENTS" | bc -l)
    delta_lat=$(echo "$DISTANCE / $LAT_METERS * s($angle)" | bc -l)
    delta_lon=$(echo "$DISTANCE / ($LAT_METERS * c($CENTER_LAT * 3.14159265359/180)) * c($angle)" | bc -l)
    LAT=$(echo "$CENTER_LAT + $delta_lat" | bc -l)
    LON=$(echo "$CENTER_LON + $delta_lon" | bc -l)

    POWER_LEVEL=80

    echo "Starting $CONTAINER_NAME with AGENT_ID=$AGENT_ID on host port $HOST_PORT"
    docker compose run -d \
        --name $CONTAINER_NAME \
        -e AGENT_ID=$AGENT_ID \
        -e LAT=$LAT \
        -e LON=$LON \
        -e POWER_LEVEL=$POWER_LEVEL \
        -p ${HOST_PORT}:${CONTAINER_PORT} \
        $SERVICE_NAME
done

echo "Starting CLI container $CLI_CONTAINER_NAME"
docker compose up -d cli
