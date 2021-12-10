FROM rust:1.56-alpine

# install dependencies
RUN apk update \
    && apk add git bash vim make cmake autoconf sqlite-dev libuv-dev gcc automake libtool musl-dev curl cmake libuv openssl openssl-dev \
    && apk add --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing raft-dev
RUN git clone --branch c_client https://github.com/ardhipoetra/dqlite
WORKDIR dqlite

# compile dqlite
RUN autoreconf -i \
    && ./configure \
    && make -j8 install

# adjust header file (?)
RUN mkdir -p /usr/local/include/dqlite/lib \
    && cp src/*.h /usr/local/include/dqlite/ \
    && cp src/lib/*.h /usr/local/include/dqlite/lib/

# adjust header file-2 (?)
RUN sed -i 's/..\/..\/include\///g' /usr/local/include/dqlite/lib/serialize.h \
    && sed -i 's/..\/..\/include\///g' /usr/local/include/dqlite/lib/registry.h

# install the library (c client)
ADD lib/ /root/lib/
WORKDIR /root/lib
RUN make install


WORKDIR /root/lld-leasing
ENV CARGO_TERM_COLOR always
ENV LD_LIBRARY_PATH /usr/local/lib
ENV PATH /usr/local/cargo/bin:$PATH
ENV RUST_BACKTRACE 1

RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN mkdir lld-benchmark/
RUN mkdir lld-benchmark/src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > lld-benchmark/src/main.rs
COPY lld-benchmark/Cargo.toml ./lld-benchmark/
RUN mkdir lld-client/
RUN mkdir lld-client/src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > lld-client/src/main.rs
COPY lld-client/Cargo.toml ./lld-client/
RUN mkdir lld-common/
RUN mkdir lld-common/src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > lld-common/src/main.rs
COPY lld-common/Cargo.toml ./lld-common/
RUN mkdir lld-server/
RUN mkdir lld-server/src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > lld-server/src/main.rs
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > lld-server/build.rs
COPY lld-server/Cargo.toml ./lld-server/
WORKDIR /root/lld-leasing/lld-server
RUN cargo install --path . --locked
WORKDIR /root/lld-leasing
RUN rm -rf ./target

COPY .cargo/ .cargo/
COPY Cargo.lock ./
COPY Cargo.toml ./
COPY src/ src/
COPY lld-common/ lld-common/
COPY lld-server/ lld-server/
COPY lld-client/ lld-client/
COPY lld-benchmark/ lld-benchmark/
WORKDIR /root/lld-leasing/lld-server
RUN cargo install --path . --locked
WORKDIR /root/lld-leasing
COPY ips.csv ./

CMD /usr/local/cargo/bin/lld-server

EXPOSE 3030
EXPOSE 3040
