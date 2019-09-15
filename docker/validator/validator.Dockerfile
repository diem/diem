FROM debian:stretch as builder

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN echo "deb http://deb.debian.org/debian stretch-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y protobuf-compiler/stretch-backports cmake curl \
    && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY . /libra
RUN cargo build --release -p libra_node -p client -p benchmark

### Production Image ###
FROM debian:stretch

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY docker/validator/install-tools.sh /root
COPY --from=builder /libra/target/release/libra_node /opt/libra/bin

# Admission control
EXPOSE 8000
# Validator network
EXPOSE 6180
# Metrics
EXPOSE 9101

# Capture backtrace on error
ENV RUST_BACKTRACE 1

# Define SEED_PEERS, NODE_CONFIG, NETWORK_KEYPAIRS, CONSENSUS_KEYPAIR, GENESIS_BLOB and PEER_ID environment variables when running
CMD cd /opt/libra/etc && echo "$NODE_CONFIG" > node.config.toml && echo "$SEED_PEERS" > seed_peers.config.toml && echo "$NETWORK_KEYPAIRS" > network_keypairs.config.toml && echo "$CONSENSUS_KEYPAIR" > consensus_keypair.config.toml && echo "$GENESIS_BLOB" | base64 -d > genesis.blob && exec /opt/libra/bin/libra_node -f node.config.toml --peer_id "$PEER_ID"

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
