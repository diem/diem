FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY cargo-toolchain /libra/cargo-toolchain
RUN rustup install $(cat cargo-toolchain)

FROM toolchain AS builder

COPY . /libra

RUN ./docker/build-common.sh

### Production Image ###
FROM debian:buster-slim AS prod

RUN echo "deb http://deb.debian.org/debian bullseye main" > /etc/apt/sources.list.d/bullseye.list && \
    echo "Package: *\nPin: release n=bullseye\nPin-Priority: 50" > /etc/apt/preferences.d/bullseye

RUN apt-get update && \
    apt-get -y install socat awscli/bullseye && \
    apt-get clean && \
    rm -r /var/lib/apt/lists/*

COPY --from=builder /libra/target/release/libra-genesis-tool /usr/local/bin
COPY --from=builder /libra/target/release/libra-operational-tool /usr/local/bin
COPY --from=builder /libra/target/release/db-bootstrapper /usr/local/bin
COPY --from=builder /libra/target/release/db-backup /usr/local/bin
COPY --from=builder /libra/target/release/db-restore /usr/local/bin

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
