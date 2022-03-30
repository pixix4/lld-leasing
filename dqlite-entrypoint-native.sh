#!/bin/sh

echo "Starting dqlite server at 0.0.0.0:24000"
echo "Starting dqlite server at 0.0.0.0:25000"
echo "Starting dqlite server at 0.0.0.0:26000"

SERVER_ADDRESS=0.0.0.0 NODE_ID=1 PORT=24000 /root/server > /root/1.log 2>/root/1.err &
SERVER_ADDRESS=0.0.0.0 NODE_ID=2 PORT=25000 /root/server > /root/2.log 2>/root/2.err &
SERVER_ADDRESS=0.0.0.0 NODE_ID=3 PORT=26000 /root/server > /root/3.log 2>/root/3.err &
wait

trap 'echo "Shutdown dqlite servers..." && pkill server && rm -rf /root/*.log && rm -rf /root/*.err && rm -rf /tmp/dqlite-rs*' SIGTERM
