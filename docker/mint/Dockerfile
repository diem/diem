FROM debian:buster-20200803@sha256:a44ab0cca6cd9411032d180bc396f19bc98f71972d2398d50460145cab81c5ab AS debian-base

FROM debian-base AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git pkg-config libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY cargo-toolchain /libra/cargo-toolchain
RUN rustup install $(cat cargo-toolchain)

FROM toolchain AS builder

ARG ENABLE_FAILPOINTS
COPY . /libra

RUN ./docker/build-common.sh

### Production Image ###
FROM debian-base AS pre-test

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update && apt-get install -y python3-pip nano net-tools tcpdump iproute2 netcat \
    && apt-get clean && rm -r /var/lib/apt/lists/*

# RUN apt-get install python3
COPY docker/mint/requirements.txt /libra/docker/mint/requirements.txt
RUN pip3 install -r /libra/docker/mint/requirements.txt

RUN mkdir -p /opt/libra/bin  /libra/client/data/wallet/

COPY --from=builder /libra/target/release/cli /opt/libra/bin
COPY --from=builder /libra/target/release/libra-faucet /opt/libra/bin
COPY docker/mint/server.py /opt/libra/bin

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
