#!/bin/bash

./start_dqlite.sh

sleep 1

./target/debug/server

sleep 1

./stop_dqlite.sh
