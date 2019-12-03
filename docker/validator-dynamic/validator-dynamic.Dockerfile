FROM debian:buster as config_builder

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

COPY . /libra

RUN cargo build --release -p dynamic-config-builder && cd target/release && rm -r build deps incremental

### Production Image ###
FROM libra_e2e:latest as validator_with_config
COPY --from=config_builder /libra/target/release/dynamic-config-builder /opt/libra/bin
COPY docker/validator-dynamic/docker-run-dynamic.sh /
CMD /docker-run-dynamic.sh
