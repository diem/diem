FROM debian:buster-20211011@sha256:f9182ead292f45165f4a851e5ff98ea0800e172ccedce7d17764ffaae5ed4d6e AS debian-base

FROM debian-base AS toolchain

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

FROM debian-base

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
