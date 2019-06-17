FROM debian:stretch as builder

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN echo "deb http://deb.debian.org/debian stretch-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y protobuf-compiler/stretch-backports cmake golang curl \
    && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

COPY . /libra
RUN cargo build -p client --release

### Production Image ###
FROM debian:stretch

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update && apt-get install -y python3-pip nano net-tools tcpdump iproute2 netcat \
    && apt-get clean && rm -r /var/lib/apt/lists/*

# RUN apt-get install python3
# TODO: Move to requirements.txt
RUN pip3 install flask flask_limiter gunicorn pexpect

RUN mkdir -p /opt/libra/bin /opt/libra/etc /libra/client/data/wallet/

#TODO: Remove this once wallet location is set properly
RUN mkdir -p /libra/client/data/wallet/

COPY --from=builder /libra/target/release/client /opt/libra/bin
COPY docker/mint/server.py /opt/libra/bin

# Mint proxy listening address
EXPOSE 8000

# Define TRUSTED_PEERS, MINT_KEY, AC_HOST and AC_PORT environment variables when running
CMD cd /opt/libra/etc && echo "$TRUSTED_PEERS" > trusted_peers.config.toml && echo "$MINT_KEY" | \
    base64 -d > mint.key && \
    cd /opt/libra/bin && \
    gunicorn --bind 0.0.0.0:8000 --access-logfile - --error-logfile - --log-level $LOG_LEVEL server


ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
