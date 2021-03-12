FROM debian:buster-20210311@sha256:9d4ab94af82b2567c272c7f47fa1204cd9b40914704213f1c257c44042f82aac AS toolchain

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

### Pre-Production Image ###
FROM debian:buster-20210311@sha256:9d4ab94af82b2567c272c7f47fa1204cd9b40914704213f1c257c44042f82aac AS pre-prod

RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && apt-get clean && rm -r /var/lib/apt/lists/*

RUN mkdir -p /opt/diem/bin /opt/diem/etc
COPY --from=builder /diem/target/release/cli /opt/diem/bin/diem_client

### Test pre-production Image ###
FROM pre-prod as testing

RUN /opt/diem/bin/diem_client -h

### Production Image ###
FROM pre-prod AS prod

ENTRYPOINT ["/opt/diem/bin/diem_client"]
CMD ["--url", "https://testnet.diem.com/v1", "--waypoint_url",  "https://testnet.diem.com/waypoint.txt", "--chain-id", "TESTNET"]

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
