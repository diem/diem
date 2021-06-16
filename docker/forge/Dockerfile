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

RUN IMAGE_TARGETS="test" ./docker/build-common.sh

FROM debian:buster-20210511@sha256:acf7795dc91df17e10effee064bd229580a9c34213b4dba578d64768af5d8c51

RUN apt-get update && apt-get install -y libssl1.1 openssh-client wget && apt-get clean && rm -r /var/lib/apt/lists/*

RUN mkdir /diem
COPY rust-toolchain /diem/rust-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh
RUN /diem/scripts/dev_setup.sh -b -p -i helm -i kubectl
ENV PATH "$PATH:/root/bin"

RUN mkdir /etc/forge
WORKDIR /etc/forge
COPY --from=builder /diem/target/release/forge /usr/local/bin/forge
ENTRYPOINT ["forge"]
ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
