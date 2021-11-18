FROM rust:1.56-bullseye

# install dependencies
RUN apt update \
    && apt install -y git bash vim make clang cmake autoconf sqlite3 libsqlite3-0 libsqlite3-dev libuv1-dev gcc automake libtool libraft-dev curl
RUN git clone --branch c_client https://github.com/ardhipoetra/dqlite
WORKDIR dqlite

RUN rustup component add clippy
RUN rustup component add rustfmt

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

# prepare dqlite server program
ADD run_server.c /root/
WORKDIR /root/
RUN gcc run_server.c -ldqlite -lraft -lsqlite3 -o /root/server

EXPOSE 24000
EXPOSE 25000
EXPOSE 26000

EXPOSE 3030
EXPOSE 3040

CMD /bin/bash
COPY exec.sh ./
COPY ips.csv ./

WORKDIR /root/lld-leasing
ENV CARGO_TERM_COLOR always
ENV LD_LIBRARY_PATH /usr/local/lib
ENV PATH /usr/local/cargo/bin:$PATH
ENV RUST_BACKTRACE 1

RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > build.rs
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo install --path . --locked
RUN rm -rf ./target

COPY src/ src/
COPY build.rs ./
RUN cargo install --features dqlite --bin lld_leasing --path . --locked
RUN cargo install --features dqlite --bin client --path . --locked
RUN cargo install --features dqlite --bin benchmark --path . --locked

WORKDIR /root
