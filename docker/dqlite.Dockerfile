FROM alpine:3.14

# install dependencies
RUN apk update \
    && apk add git bash vim make cmake autoconf sqlite-dev libuv-dev gcc automake libtool musl-dev curl \
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

# prepare dqlite server program
ADD run_server.c /root/
WORKDIR /root/
RUN gcc run_server.c -ldqlite -lraft -lsqlite3 -o /root/server

EXPOSE 24000
EXPOSE 25000
EXPOSE 26000

COPY ./dqlite-entrypoint.sh /
ENTRYPOINT ["/dqlite-entrypoint.sh"]
