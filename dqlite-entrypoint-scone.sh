#!/bin/sh

echo "Starting dqlite server at 172.20.0.11:24000"
echo "Starting dqlite server at 172.20.0.11:25000"
echo "Starting dqlite server at 172.20.0.11:26000"

SCONE_LOG=DEBUG SCONE_LAS_ADDR=las SCONE_CAS_ADDR=$SCONE_CAS_ADDR SCONE_CONFIG_ID=$LLD_SESSION/dqlite1 /root/server > /root/1.log &
SCONE_LOG=DEBUG SCONE_LAS_ADDR=las SCONE_CAS_ADDR=$SCONE_CAS_ADDR SCONE_CONFIG_ID=$LLD_SESSION/dqlite2 /root/server > /root/2.log &
SCONE_LOG=DEBUG SCONE_LAS_ADDR=las SCONE_CAS_ADDR=$SCONE_CAS_ADDR SCONE_CONFIG_ID=$LLD_SESSION/dqlite3 /root/server > /root/3.log

trap 'echo "Shutdown dqlite servers..." && pkill server && rm -rf /root/*.log && rm -rf /tmp/dqlite-rs*' SIGTERM
