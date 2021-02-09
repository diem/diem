FROM debian:buster-20210208@sha256:1092695e843ad975267131f27a2b523128c4e03d2d96574bbdd7cf949ed51475 AS toolchain

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
FROM debian:buster-20210111-slim@sha256:b1af07039fe341833982bae85a2724ac8600ec5c74c37277c7a6ef7cddfb2cd0 AS pre-prod

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
