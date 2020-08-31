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

### Production Image ###
FROM debian:buster-20200803@sha256:a44ab0cca6cd9411032d180bc396f19bc98f71972d2398d50460145cab81c5ab AS prod

RUN addgroup --system --gid 6180 libra && adduser --system --ingroup libra --no-create-home --uid 6180 libra

RUN mkdir -p /opt/libra/bin /opt/libra/etc /opt/libra/data

COPY docker/safety-rules/key-manager.sh /
COPY docker/safety-rules/safety-rules.sh /

COPY --from=builder /libra/target/release/config-builder /opt/libra/bin
COPY --from=builder /libra/target/release/libra-key-manager /opt/libra/bin
COPY --from=builder /libra/target/release/safety-rules /opt/libra/bin

ENV RUST_BACKTRACE 1
CMD /safety-rules.sh

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
