FROM debian:buster-20210511@sha256:acf7795dc91df17e10effee064bd229580a9c34213b4dba578d64768af5d8c51 AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git pkg-config libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /diem
COPY rust-toolchain /diem/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS builder

ARG ENABLE_FAILPOINTS
COPY . /diem

RUN IMAGE_TARGETS="release" ./docker/build-common.sh

### Production Image ###
FROM debian:buster-20210511@sha256:acf7795dc91df17e10effee064bd229580a9c34213b4dba578d64768af5d8c51  AS pre-prod

RUN apt-get update && apt-get -y install libssl1.1 wget busybox gettext-base && apt-get clean && rm -r /var/lib/apt/lists/*
RUN cd /usr/local/bin && wget "https://storage.googleapis.com/kubernetes-release/release/v1.18.6/bin/linux/amd64/kubectl" -O kubectl && chmod +x kubectl
RUN cd /usr/local/bin && wget "https://releases.hashicorp.com/vault/1.5.0/vault_1.5.0_linux_amd64.zip" -O- | busybox unzip - && chmod +x vault

RUN mkdir -p /opt/diem/bin
COPY --from=builder /diem/target/release/diem-genesis-tool /usr/local/bin
COPY --from=builder /diem/target/release/diem-operational-tool /usr/local/bin

FROM pre-prod as testing

RUN /usr/local/bin/diem-genesis-tool -h
RUN /usr/local/bin/diem-operational-tool -h

FROM pre-prod as prod

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
