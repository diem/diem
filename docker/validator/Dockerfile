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
FROM debian:buster-20210511@sha256:acf7795dc91df17e10effee064bd229580a9c34213b4dba578d64768af5d8c51 AS prod

RUN apt-get update && apt-get install -y libssl1.1 && apt-get clean && rm -r /var/lib/apt/lists/*

RUN addgroup --system --gid 6180 diem && adduser --system --ingroup diem --no-create-home --uid 6180 diem

RUN mkdir -p /opt/diem/bin /opt/diem/etc
COPY docker/install-tools.sh /root
COPY --from=builder /diem/target/release/diem-node /opt/diem/bin
COPY --from=builder /diem/target/release/db-backup /opt/diem/bin
COPY --from=builder /diem/target/release/db-bootstrapper /opt/diem/bin
COPY --from=builder /diem/target/release/db-restore /opt/diem/bin

# Admission control
EXPOSE 8000
# Validator network
EXPOSE 6180
# Metrics
EXPOSE 9101
# Backup
EXPOSE 6186

# Capture backtrace on error
ENV RUST_BACKTRACE 1

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
