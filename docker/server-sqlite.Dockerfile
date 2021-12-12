FROM registry.scontain.com:5050/sconecuratedimages/crosscompilers:ubuntu

# install dependencies
RUN apt update \
    && apt install -y curl autoconf libtool make

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

EXPOSE 3030
EXPOSE 3040

COPY . .

WORKDIR /root/lld-leasing/lld-server
RUN scone cargo install --path . --locked --target=x86_64-scone-linux-musl
WORKDIR /root/lld-leasing

CMD /root/.cargo/bin/lld-server
