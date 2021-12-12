#!/bin/bash

echo "Starting dqlite server at 0.0.0.0:24000"
echo "Starting dqlite server at 0.0.0.0:25000"
echo "Starting dqlite server at 0.0.0.0:26000"

SERVER_ADDRESS=0.0.0.0 NODE_ID=1 PORT=24000 /root/server > /root/1.log &
SERVER_ADDRESS=0.0.0.0 NODE_ID=2 PORT=25000 /root/server > /root/2.log &
SERVER_ADDRESS=0.0.0.0 NODE_ID=3 PORT=26000 /root/server > /root/3.log

trap 'echo "Shutdown dqlite servers..." && pkill server && rm -rf /root/*.log && rm -rf /tmp/dqlite-rs*' SIGTERM
