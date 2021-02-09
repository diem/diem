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

FROM debian:buster-20210208@sha256:1092695e843ad975267131f27a2b523128c4e03d2d96574bbdd7cf949ed51475

RUN apt-get update && apt-get install -y libssl1.1 openssh-client wget && apt-get clean && rm -r /var/lib/apt/lists/*
RUN cd /usr/local/bin && wget "https://storage.googleapis.com/kubernetes-release/release/v1.17.0/bin/linux/amd64/kubectl" -O kubectl && chmod +x kubectl
RUN mkdir /etc/cluster-test
WORKDIR /etc/cluster-test
COPY --from=builder /diem/target/release/cluster-test /usr/local/bin/cluster-test
ENTRYPOINT ["cluster-test"]
ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
