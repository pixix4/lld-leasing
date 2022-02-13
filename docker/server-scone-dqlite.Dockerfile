FROM registry.scontain.com:5050/sconecuratedimages/crosscompilers:ubuntu as builder

# install dependencies
RUN apt update \
    && apt install -y curl autoconf libtool make git pkg-config

# build sqlite
WORKDIR /root
RUN curl https://www.sqlite.org/2021/sqlite-autoconf-3370000.tar.gz > sqlite-autoconf-3370000.tar.gz \
    && tar xvzf sqlite-autoconf-3370000.tar.gz \
    && cd sqlite-autoconf-3370000 \
    && autoreconf -i \
    && ./configure \
    && make && make install

# build libuv
WORKDIR /root
RUN git clone https://github.com/libuv/libuv.git \
    && cd libuv \
    &&  sh autogen.sh \
    && ./configure \
    && make && make install

# build LZ4
WORKDIR /root
RUN git clone https://github.com/lz4/lz4.git \
    && cd lz4 \
    && make && make install

# build raft
WORKDIR /root
RUN git clone https://github.com/canonical/raft.git \
    && cd raft \
    && autoreconf -i \
    && ./configure \
    && make && make install

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

# build openssl
WORKDIR /root
RUN curl https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-3.4.2.tar.gz > libressl-3.4.2.tar.gz \
    && tar xvzf libressl-3.4.2.tar.gz \
    && cd libressl-3.4.2 \
    && ./configure --enable-shared=no \
    && make \
    && make install

WORKDIR /root/lld-leasing

COPY . .

WORKDIR /root/lld-leasing/lld-server

ENV OPENSSL_STATIC 1
ENV PKG_CONFIG_ALLOW_CROSS 1
RUN scone cargo install --path . --locked --features dqlite --target=x86_64-scone-linux-musl

FROM alpine:3.14

ENV RUST_BACKTRACE 1
ENV RUST_LOG info
EXPOSE 3030
EXPOSE 3040

ENTRYPOINT [ "/usr/local/bin/lld-server" ]

COPY ./ips.csv ./
COPY --from=builder /root/.cargo/bin/lld-server /usr/local/bin
