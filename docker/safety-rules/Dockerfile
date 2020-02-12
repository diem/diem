FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS builder

COPY . /libra

RUN cargo build --release -p libra-node -p cli -p config-builder -p safety-rules && cd target/release && rm -r build deps incremental
RUN strip /libra/target/release/config-builder

### Production Image ###
FROM debian:buster AS prod

RUN addgroup --system --gid 6180 libra && adduser --system --ingroup libra --no-create-home --uid 6180 libra

RUN mkdir -p /opt/libra/bin /opt/libra/etc /opt/libra/data/common
COPY docker/safety-rules/docker-run-dynamic.sh /
COPY --from=builder /libra/target/release/safety-rules /opt/libra/bin
COPY --from=builder /libra/target/release/config-builder /opt/libra/bin
COPY --from=builder /libra/language/stdlib/modules /libra/language/stdlib/modules

ENV RUST_BACKTRACE 1
CMD /docker-run-dynamic.sh

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
