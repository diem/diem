FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y protobuf-compiler/buster cmake curl clang git \
    && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS builder

COPY . /libra

RUN cargo build --release -p libra-node -p client -p benchmark && cd target/release && rm -r build deps incremental

### Production Image ###
FROM debian:buster AS prod

RUN mkdir -p /opt/libra/bin /opt/libra/etc

COPY --from=builder /libra/target/release/ruben /opt/libra/bin
COPY docker/bench/bench_init.sh /opt/libra/bin/
RUN chmod +x /opt/libra/bin/bench_init.sh

# Metrics
EXPOSE 9101

# Define MINT_KEY, AC_HOST and AC_DEBUG environment variables when running
ENTRYPOINT ["/opt/libra/bin/bench_init.sh"]

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
