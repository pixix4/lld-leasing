FROM rust:1.54-alpine3.14 as builder

# install dependencies
RUN apk add --no-cache gcc musl-dev sqlite-static openssl-dev openssl-libs-static pkgconf

WORKDIR /root/lld-leasing

COPY . .

WORKDIR /root/lld-leasing/lld-server
RUN cargo install --path . --locked

FROM alpine:3.14

ENV RUST_BACKTRACE 1
ENV RUST_LOG info
EXPOSE 3030
EXPOSE 3040

ENTRYPOINT [ "/usr/local/bin/lld-server" ]

COPY ./ips.csv ./
COPY --from=builder /usr/local/cargo/bin/lld-server /usr/local/bin
