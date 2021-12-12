# lld leasing

## Build

```bash
% Build dqlite server image
docker build -t pixix4/dqlite:latest -f docker/dqlite.Dockerfile .

% Build leasing server image
docker build -t pixix4/lld-native-dqlite:latest -f docker/server-native-dqlite.Dockerfile .
% or with sgx/scone
docker build -t pixix4/lld-scone-dqlite:latest -f docker/server-scone-dqlite.Dockerfile .
```

## Run

```bash
% Start dqlite server
docker run --rm -it -p 24000:24000 -p 25000:25000 -p 26000:26000 pixix4/dqlite:latest

% Start leasing server
docker run --rm -it -p 3030:3030 -p 3040:3040 pixix4/lld-native-dqlite:latest
% or with sgx/scone
docker run --rm -it -p 3030:3030 -p 3040:3040 pixix4/lld-scone-dqlite:latest

% Start client
cargo run --release -p lld-client -- "application-id"
```

## Benchmark

```bash
cargo run --release -p lld-benchmark -- --max 4 --repeat 2 --container NativeDqlite > logs/benchmark.csv
python3 benchmark.py
```

Usage of `lld-benchmark`:
```
USAGE:
    lld-benchmark [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --container <container>     [env: LLD_CONTAINER=]  [possible values: NativeSqlite, NativeDqlite, SconeSqlite,
                                   SconeDqlite]
    -d, --duration <duration>       [env: LLD_DURATION=]
        --build <force_build>       [env: LLD_FORCE_BUILD=]
        --http_uri <http_uri>       [env: LLD_HTTP_URI=]
    -m, --max <max>                 [env: LLD_MAX=]
    -r, --repeat <repeat>           [env: LLD_REPEAT=]
        --tcp_uri <tcp_uri>         [env: LLD_TCP_URI=]

SUBCOMMANDS:
    build    Builds the docker images without running the tests
    help     Prints this message or the help of the given subcommand(s)
```

A benchmark round creates `N` clients that continuously send leasing requests for 3 seconds. 2 clients each use the same `application id` with different `instance id`s. Thus there are `N/2` clients with granted leases and `N/2` clients with rejected leases. Leasings requests timeout after 1 second.

### Use sqlite file as database

Average response time of granted leases relative to the number of concurrent clients:

![Average response time of granted leases relative to the number of concurrent clients](images/bench_0/response-time-granted.png "Average response time of granted leases relative to the number of concurrent clients")

Average response time of rejected leases relative to the number of concurrent clients:

![Average response time of rejected leases relative to the number of concurrent clients](images/bench_0/response-time-rejected.png "Average response time of rejected leases relative to the number of concurrent clients")

Number of timeouts relative to the number of concurrent clients:

![Number of timeouts relative to the number of concurrent clients](images/bench_0/response-count-timeout.png "Number of timeouts relative to the number of concurrent clients")

### Simulate dqlite network delay with a 10ms sleep before each sqlite request

Average response time of granted leases relative to the number of concurrent clients:

![Average response time of granted leases relative to the number of concurrent clients](images/bench_1/response-time-granted.png "Average response time of granted leases relative to the number of concurrent clients")

Average response time of rejected leases relative to the number of concurrent clients:

![Average response time of rejected leases relative to the number of concurrent clients](images/bench_1/response-time-rejected.png "Average response time of rejected leases relative to the number of concurrent clients")

Number of timeouts relative to the number of concurrent clients:

![Number of timeouts relative to the number of concurrent clients](images/bench_1/response-count-timeout.png "Number of timeouts relative to the number of concurrent clients")

### Use local dqlite cluster with 3 servers as database

Average response time of granted leases relative to the number of concurrent clients:

![Average response time of granted leases relative to the number of concurrent clients](images/bench_2/response-time-granted.png "Average response time of granted leases relative to the number of concurrent clients")

Average response time of rejected leases relative to the number of concurrent clients:

![Average response time of rejected leases relative to the number of concurrent clients](images/bench_2/response-time-rejected.png "Average response time of rejected leases relative to the number of concurrent clients")

Number of timeouts relative to the number of concurrent clients:

![Number of timeouts relative to the number of concurrent clients](images/bench_2/response-count-timeout.png "Number of timeouts relative to the number of concurrent clients")
