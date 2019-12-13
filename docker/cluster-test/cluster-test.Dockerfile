## DO NOT EDIT cluster-test.Dockerfile
## Edit cluster-test.Dockerfile.template instead and use ./generate.sh to generate cluster-test.Dockerfile
## Using C preprocessor to compile this template into cluster-test.Dockerfile for one-step build
## and cluster-test.container.Dockerfile for incremental build
##
## This adds some restrictions to this template file
## - To comment must use double ##, single # will be treated as pre-processor command
## -
FROM debian:buster AS builder
## To use http/https proxy while building, use:
## docker build --build-arg https_proxy=http:
RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list && apt-get update && apt-get install -y protobuf-compiler/buster cmake curl clang git
RUN curl --proto '=https' --tlsv1.2 -sSf "https://sh.rustup.rs" | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"
WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)
COPY . /libra
RUN docker/cluster-test/compile.sh
FROM debian:buster
RUN apt-get update && apt-get install -y openssh-client
RUN mkdir /etc/cluster-test
WORKDIR /etc/cluster-test
COPY --from=builder /target/release/cluster-test /usr/local/bin/cluster-test
ENTRYPOINT ["cluster-test"]
ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
