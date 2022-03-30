FROM rust:1.54-alpine3.14 as builder

# install dependencies
RUN apk add --no-cache gcc musl-dev autoconf automake libtool make git openssl-dev openssl-libs-static pkgconf

WORKDIR /root/lld-leasing

COPY . .

WORKDIR /root/lld-leasing/lld-client
RUN cargo install --path . --locked

FROM alpine:3.14

ENV RUST_BACKTRACE 1
ENV RUST_LOG info

ENTRYPOINT [ "/usr/local/bin/lld-client" ]

COPY --from=builder /usr/local/cargo/bin/lld-client /usr/local/bin
