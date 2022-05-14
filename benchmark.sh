#!/bin/bash

docker build -f docker/server-native-sqlite.Dockerfile -t pixix4/server-native-sqlite:latest .

docker build -f docker/server-native-dqlite.Dockerfile -t pixix4/server-native-dqlite:latest .
docker build -f docker/native-dqlite.Dockerfile -t pixix4/native-dqlite:latest .

cargo run --release -p lld-benchmark | tee benchmark.csv
