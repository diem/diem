# Create a base image for mint, validator, client
FROM debian:stretch as builder

RUN echo "deb http://deb.debian.org/debian stretch-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update \
    && apt-get install -y protobuf-compiler/stretch-backports cmake curl clang \
    && apt-get clean \
    && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY . /libra
RUN nproc && free
RUN cargo build --release -p libra-node -p client -p benchmark -j 16
