### Build layer
ARG TOOLCHAIN_TAG
FROM toolchain_rust:$TOOLCHAIN_TAG AS builder

WORKDIR /libra
COPY . /libra

RUN cargo build --release -p libra-node -p cli -p config-builder -p safety-rules && cd target/release && rm -r build deps incremental

### Production Image ###
FROM debian:buster AS prod

RUN apt-get update \
    && apt-get -y install gettext-base=0.19.8.1-9 \
    && apt-get clean \
    && rm -r /var/lib/apt/lists/*
COPY --from=builder /staging/bin/kubectl /usr/local/bin
RUN chmod a+x /usr/local/bin/kubectl

RUN mkdir -p /opt/libra/bin
COPY --from=builder /libra/target/release/config-builder /opt/libra/bin

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
