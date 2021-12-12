FROM registry.scontain.com:5050/sconecuratedimages/crosscompilers:ubuntu

# install dependencies
RUN apt update \
    && apt install -y git bash vim make clang cmake autoconf libuv1-dev gcc automake libtool curl software-properties-common pkg-config \
    && add-apt-repository ppa:dqlite/dev \
    && apt update \
    && apt install -y libraft-dev

# build sqlite
WORKDIR /root
RUN curl https://www.sqlite.org/2021/sqlite-autoconf-3370000.tar.gz > sqlite-autoconf-3370000.tar.gz \
    && tar xvzf sqlite-autoconf-3370000.tar.gz \
    && cd /root/sqlite-autoconf-3370000 \
    && autoreconf -i \
    && ./configure \
    && make \
    && make install

WORKDIR /root/lld-leasing
ENV CARGO_TERM_COLOR always
ENV PATH /usr/local/cargo/bin:$PATH
ENV RUST_BACKTRACE 1
ENV RUST_LOG info
