#!/bin/bash

./start_dqlite.sh

sleep 1

./target/debug/lld_leasing

sleep 1

./stop_dqlite.sh
