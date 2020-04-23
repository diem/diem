### Toolchain layer
FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update \
    && apt-get install -y \
        cmake=3.13.4-1 \
        curl=7.64.0-4+deb10u1 \
        clang=1:7.0-47 \
        git=1:2.20.1-2+deb10u1 \
        wget=1.20.1-1.1 \
    && apt-get clean \
    && rm -r /var/lib/apt/lists/*
# 3rd party bin
RUN mkdir -p /staging/bin && \
    cd /staging/bin && \
    wget https://storage.googleapis.com/kubernetes-release/release/v1.17.0/bin/linux/amd64/kubectl -O kubectl

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /staging/libra
COPY . /staging/libra
COPY rust-toolchain /staging/rust-toolchain
RUN rustup install $(cat rust-toolchain)

# Prefetch crates
RUN cargo fetch
RUN set -e; shasum -a 256 Cargo.lock | cut -d' ' -f1 > /staging/crates_cache.sha256
WORKDIR /staging
RUN rm -rf /staging/libra
