FROM debian:buster-20210111@sha256:a5edb9fa5b2a8d6665ed911466827734795ef10d2b3985a46b7e9c7f0161a6b3 AS debian-base

FROM debian-base AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git pkg-config libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /diem
COPY rust-toolchain /diem/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY cargo-toolchain /diem/cargo-toolchain
RUN rustup install $(cat cargo-toolchain)

FROM toolchain AS builder

ARG ENABLE_FAILPOINTS
COPY . /diem

RUN ./docker/build-common.sh

### Production Image ###
FROM debian-base AS pre-test

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update && apt-get install -y libssl1.1 python3-pip nano net-tools tcpdump iproute2 netcat \
    && apt-get clean && rm -r /var/lib/apt/lists/*

# RUN apt-get install python3
COPY docker/mint/requirements.txt /diem/docker/mint/requirements.txt
RUN pip3 install -r /diem/docker/mint/requirements.txt

RUN mkdir -p /opt/diem/bin  /diem/client/data/wallet/

COPY --from=builder /diem/target/release/cli /opt/diem/bin
COPY --from=builder /diem/target/release/diem-faucet /opt/diem/bin
COPY docker/mint/server.py /opt/diem/bin

# Test the docker container before shipping.
FROM pre-test AS test

#install needed tools
RUN apt-get update && apt-get install -y procps

FROM pre-test as prod
# Mint proxy listening address
EXPOSE 8000

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
