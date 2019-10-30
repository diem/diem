FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y protobuf-compiler/buster cmake curl clang \
    && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS builder

COPY . /libra

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

RUN cargo build --release -p libra-node -p client -p benchmark && cd target/release && rm -r build deps incremental

### Production Image ###
FROM debian:buster AS prod

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY docker/install-tools.sh /root
COPY --from=builder /libra/target/release/libra-node /opt/libra/bin

# Admission control
EXPOSE 8000
# Validator network
EXPOSE 6180
# Metrics
EXPOSE 9101

# Capture backtrace on error
ENV RUST_BACKTRACE 1

# Define SEED_PEERS, NODE_CONFIG, NETWORK_KEYPAIRS, CONSENSUS_KEYPAIR, GENESIS_BLOB and PEER_ID environment variables when running
CMD cd /opt/libra/etc \
    && echo "$NODE_CONFIG" > node.config.toml \
    && echo "$SEED_PEERS" > seed_peers.config.toml \
    && echo "$NETWORK_KEYPAIRS" > network_keypairs.config.toml \
    && echo "$CONSENSUS_KEYPAIR" > consensus_keypair.config.toml \
    && echo "$FULLNODE_KEYPAIRS" > fullnode_keypairs.config.toml \
    && exec /opt/libra/bin/libra-node -f node.config.toml

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
