# lld leasing

## Build

```bash
docker build -t pixix4/lld:latest .
```

## Run

```bash
docker run --rm pixix4/lld:latest lld_leasing
docker run --rm pixix4/lld:latest client "application-id"
```

## Benchmark

```bash
docker run --rm pixix4/lld:latest benchmark --max=10 --repeat=4
```
