FROM rust:1.56-slim as builder

WORKDIR /usr/src/builder
ENV CARGO_TERM_COLOR always

RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo install --path . --locked

COPY src/ src/
RUN cargo install --bin benchmark --path . --locked
RUN cargo install --bin client --path . --locked
RUN cargo install --bin lld_leasing --path . --locked

FROM debian:buster-slim
RUN mkdir /opt/bin
WORKDIR /opt/bin
ENV PATH /opt/bin:$PATH
EXPOSE 3030

COPY --from=builder /usr/local/cargo/bin/benchmark /opt/bin/
COPY --from=builder /usr/local/cargo/bin/client /opt/bin/
COPY --from=builder /usr/local/cargo/bin/lld_leasing /opt/bin/
