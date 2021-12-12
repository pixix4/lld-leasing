FROM registry.scontain.com:5050/sconecuratedimages/crosscompilers:ubuntu as builder

# install dependencies
RUN apt update \
    && apt install -y curl autoconf libtool make

# build sqlite
WORKDIR /root
RUN curl https://www.sqlite.org/2021/sqlite-autoconf-3370000.tar.gz > sqlite-autoconf-3370000.tar.gz \
    && tar xvzf sqlite-autoconf-3370000.tar.gz \
    && cd sqlite-autoconf-3370000 \
    && autoreconf -i \
    && ./configure \
    && make \
    && make install

WORKDIR /root/lld-leasing

COPY . .

WORKDIR /root/lld-leasing/lld-server
RUN scone cargo install --path . --locked --target=x86_64-scone-linux-musl

FROM alpine:3.14

ENV RUST_BACKTRACE 1
ENV RUST_LOG info
EXPOSE 3030
EXPOSE 3040

ENTRYPOINT [ "/usr/local/bin/lld-server" ]

COPY ./ips.csv ./
COPY --from=builder /root/.cargo/bin/lld-server /usr/local/bin
