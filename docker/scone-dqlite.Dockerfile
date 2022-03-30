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
RUN git clone https://github.com/ardhipoetra/raft.git --branch rm_kaio \
    && cd raft \
    && autoreconf -i \
    && ./configure --enable-uv \
    && make && make install

# build dqlite
WORKDIR /root
RUN git clone https://github.com/ardhipoetra/dqlite --branch c_client \
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

# prepare dqlite server program
ADD run_server.c /root/
WORKDIR /root/
RUN gcc run_server.c -static -I/usr/local/include -L/usr/local/lib -ldqlite -lraft -lsqlite3 -luv -llz4 -o /root/server

FROM alpine:3.14

WORKDIR /root/
EXPOSE 24000
EXPOSE 25000
EXPOSE 26000

ENTRYPOINT ["/dqlite-entrypoint-scone.sh"]

COPY ./dqlite-entrypoint-scone.sh /
COPY --from=builder /root/server /root/server
