FROM rust:1.54-alpine3.14 as builder

# install dependencies
RUN apk update \
    && apk add gcc musl-dev sqlite-dev sqlite-static libuv-dev libuv-static autoconf automake libtool make git openssl-dev openssl-libs-static pkgconf \
    && apk add --repository=http://dl-cdn.alpinelinux.org/alpine/edge/community  raft-dev raft-static

# build dqlite
WORKDIR /root
RUN git clone --branch c_client https://github.com/ardhipoetra/dqlite \
    && cd dqlite \
    && autoreconf -i \
    && ./configure \
    && make && make install

# adjust header file (?)
WORKDIR /root/dqlite
RUN mkdir -p /usr/local/include/dqlite/lib \
    && cp src/*.h /usr/local/include/dqlite/ \
    && cp src/lib/*.h /usr/local/include/dqlite/lib/ \
    && sed -i 's/..\/..\/include\///g' /usr/local/include/dqlite/lib/serialize.h \
    && sed -i 's/..\/..\/include\///g' /usr/local/include/dqlite/lib/registry.h

# install the library (c client)
ADD lib/ /root/lib/
WORKDIR /root/lib
RUN make install

WORKDIR /root/lld-leasing

COPY . .

WORKDIR /root/lld-leasing/lld-server
RUN cargo install --path . --locked --features dqlite

FROM alpine:3.14

ENV RUST_BACKTRACE 1
ENV RUST_LOG info
EXPOSE 3030
EXPOSE 3040

ENTRYPOINT [ "/usr/local/bin/lld-server" ]

COPY ./ips.csv ./
COPY --from=builder /usr/local/cargo/bin/lld-server /usr/local/bin
