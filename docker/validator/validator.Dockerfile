FROM debian:stretch

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY docker/install-tools.sh /root
COPY libra-node-from-docker /opt/libra/bin/libra-node

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

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
