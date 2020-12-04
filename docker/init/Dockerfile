FROM debian:buster-20201117@sha256:f520e4a80b736389c3de162b8f60608d11c9fa3b2ec619bd40aabfd7e70d3455 AS toolchain

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
FROM debian:buster-20201117@sha256:f520e4a80b736389c3de162b8f60608d11c9fa3b2ec619bd40aabfd7e70d3455  AS prod

RUN apt-get update && apt-get -y install libssl1.1 wget busybox gettext-base && apt-get clean && rm -r /var/lib/apt/lists/*
RUN cd /usr/local/bin && wget "https://storage.googleapis.com/kubernetes-release/release/v1.18.6/bin/linux/amd64/kubectl" -O kubectl && chmod +x kubectl
RUN cd /usr/local/bin && wget "https://releases.hashicorp.com/vault/1.5.0/vault_1.5.0_linux_amd64.zip" -O- | busybox unzip - && chmod +x vault

RUN mkdir -p /opt/libra/bin
COPY --from=builder /libra/target/release/libra-genesis-tool /usr/local/bin
COPY --from=builder /libra/target/release/libra-operational-tool /usr/local/bin

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
