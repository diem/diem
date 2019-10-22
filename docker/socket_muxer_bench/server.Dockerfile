### Build Image ###
FROM debian:buster as builder

RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list && apt-get update && apt-get install -y protobuf-compiler/buster cmake curl && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY . /libra
RUN cargo build --release --bin socket-bench-server

### Production Image ###
FROM debian:buster

RUN mkdir -p /opt/libra/bin /opt/libra/etc

# Hack to match bench binary since it doesn't output without the build hash
COPY --from=builder /libra/target/release/socket-bench-server /opt/libra/bin

# tcp_send
EXPOSE 52720
# tcp_noise_send
EXPOSE 52721
# tcp_muxer_send
EXPOSE 52722
# tcp_noise_muxer_send
EXPOSE 52723

ENV TCP_ADDR /ip4/0.0.0.0/tcp/52720
ENV TCP_NOISE_ADDR /ip4/0.0.0.0/tcp/52721
ENV TCP_MUXER_ADDR /ip4/0.0.0.0/tcp/52722
ENV TCP_NOISE_MUXER_ADDR /ip4/0.0.0.0/tcp/52723

CMD exec /opt/libra/bin/socket-bench-server

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
