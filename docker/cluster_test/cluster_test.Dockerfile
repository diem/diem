FROM debian:stretch

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY cluster_test_docker_builder_cluster_test /opt/libra/bin/cluster_test

# Capture backtrace on error
ENV RUST_BACKTRACE 1

# Define SEED_PEERS, NODE_CONFIG, PEER_KEYPAIRS, GENESIS_BLOB and PEER_ID environment variables when running
CMD cd /opt/libra/etc && echo "$NODE_CONFIG" > node.config.toml && echo "$SEED_PEERS" > seed_peers.config.toml && echo "$NETWORK_KEYPAIRS" > network_keypairs.config.toml && echo "$CONSENSUS_KEYPAIR" > consensus_keypair.config.toml && echo "$GENESIS_BLOB" | base64 -d > genesis.blob && exec /opt/libra/bin/libra-node -f node.config.toml

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
