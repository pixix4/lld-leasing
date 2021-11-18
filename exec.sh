#!/bin/bash

SERVER_ADDRESS=127.0.0.1 NODE_ID=1 PORT=24000 /root/server > /root/1.log &
SERVER_ADDRESS=127.0.0.1 NODE_ID=2 PORT=25000 /root/server > /root/2.log &
SERVER_ADDRESS=127.0.0.1 NODE_ID=3 PORT=26000 /root/server > /root/3.log &
sleep 1 ; /usr/local/cargo/bin/lld_leasing
