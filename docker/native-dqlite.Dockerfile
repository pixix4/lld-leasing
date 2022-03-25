FROM alpine:3.14 as builder

# install dependencies
RUN apk add --no-cache git bash vim make cmake autoconf sqlite-static sqlite-dev libuv-dev libuv-static gcc automake libtool musl-dev curl file \
    && apk add --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/community raft-dev raft-static

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

# prepare dqlite server program
ADD run_server.c /root/
WORKDIR /root/
RUN gcc run_server.c -static -ldqlite -lraft -lsqlite3 -luv -o /root/server

FROM alpine:3.14

WORKDIR /root/
EXPOSE 24000
EXPOSE 25000
EXPOSE 26000

ENTRYPOINT ["/dqlite-entrypoint.sh"]

COPY ./dqlite-entrypoint.sh /
COPY --from=builder /root/server /root/server
