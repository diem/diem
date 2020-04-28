### Build layer
ARG TOOLCHAIN_TAG
FROM toolchain_rust:$TOOLCHAIN_TAG AS builder

WORKDIR /libra
COPY . /libra

RUN cargo build --release -p libra-node -p cli -p config-builder -p safety-rules && cd target/release && rm -r build deps incremental
RUN strip target/release/cli

### Production Image ###
FROM debian:buster AS prod

RUN mkdir -p /opt/libra/bin /opt/libra/etc
COPY --from=builder /libra/target/release/cli /opt/libra/bin/libra_client

ENTRYPOINT ["/opt/libra/bin/libra_client"]
CMD ["--url", "http://client.testnet.libra.org"]

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
