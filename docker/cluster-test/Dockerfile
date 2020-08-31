FROM debian:buster-20200803@sha256:a44ab0cca6cd9411032d180bc396f19bc98f71972d2398d50460145cab81c5ab AS toolchain

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

FROM debian:buster-20200803@sha256:a44ab0cca6cd9411032d180bc396f19bc98f71972d2398d50460145cab81c5ab

RUN apt-get update && apt-get install -y openssh-client wget
RUN cd /usr/local/bin && wget "https://storage.googleapis.com/kubernetes-release/release/v1.17.0/bin/linux/amd64/kubectl" -O kubectl && chmod +x kubectl
RUN mkdir /etc/cluster-test
WORKDIR /etc/cluster-test
COPY --from=builder /libra/target/release/cluster-test /usr/local/bin/cluster-test
ENTRYPOINT ["cluster-test"]
ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
