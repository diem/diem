### Base Builder Image ###
FROM libra_base as builder
RUN strip target/release/client

### Production Image ###
FROM debian:stretch

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY --from=builder /libra/target/release/client /opt/libra/bin/libra_client
COPY scripts/cli/consensus_peers.config.toml /opt/libra/etc/consensus_peers.config.toml

ENTRYPOINT ["/opt/libra/bin/libra_client"]
CMD ["--host", "ac.testnet.libra.org", "--port", "8000", "-s", "/opt/libra/etc/consensus_peers.config.toml"]

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
