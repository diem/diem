FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y protobuf-compiler/buster cmake curl clang git \
    && apt-get clean && rm -r /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS builder

COPY . /libra

RUN cargo build --release -p libra-node -p client && cd target/release && rm -r build deps incremental

### Production Image ###
FROM debian:buster AS prod

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update && apt-get install -y python3-pip nano net-tools tcpdump iproute2 netcat \
    && apt-get clean && rm -r /var/lib/apt/lists/*

# RUN apt-get install python3
COPY docker/mint/requirements.txt /libra/docker/mint/requirements.txt
RUN pip3 install -r /libra/docker/mint/requirements.txt

RUN mkdir -p /opt/libra/bin /opt/libra/etc /libra/client/data/wallet/

#TODO: Remove this once wallet location is set properly
RUN mkdir -p /libra/client/data/wallet/

COPY --from=builder /libra/target/release/client /opt/libra/bin
COPY docker/mint/server.py /opt/libra/bin

# Mint proxy listening address
EXPOSE 8000

# Define MINT_KEY, AC_HOST and AC_PORT environment variables when running
CMD cd /opt/libra/etc && echo "$MINT_KEY" | \
    base64 -d > mint.key && \
    cd /opt/libra/bin && \
    exec gunicorn --bind 0.0.0.0:8000 --access-logfile - --error-logfile - --log-level $LOG_LEVEL server

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
