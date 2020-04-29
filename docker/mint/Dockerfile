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

### Production Image ###
FROM debian:buster AS pre-test

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update && apt-get install -y python3-pip nano net-tools tcpdump iproute2 netcat \
    && apt-get clean && rm -r /var/lib/apt/lists/*

# RUN apt-get install python3
COPY docker/mint/requirements.txt /libra/docker/mint/requirements.txt
RUN pip3 install -r /libra/docker/mint/requirements.txt

RUN mkdir -p /opt/libra/bin  /libra/client/data/wallet/

COPY --from=builder /libra/target/release/cli /opt/libra/bin
COPY --from=builder /libra/target/release/config-builder /opt/libra/bin
COPY docker/mint/server.py /opt/libra/bin
COPY docker/mint/docker-run.sh /opt/libra/bin

# Test the docker container before shipping.
FROM pre-test AS test

#install needed tools
RUN apt-get update && apt-get install -y procps

# TEST docker-run.sh
#   GIVEN necessary environment arguments to run docker-run.sh
ARG AC_PORT="8080"
ARG AC_HOST="172.18.0.13"
ARG LOG_LEVEL="info"
ARG CFG_NUM_VALIDATORS=5

#   WHEN we execute docker-run.sh
#   THEN a gunicorn process is started
#
# NOTES:
#   Attempt to find the gunicorn processes (it forks itself) with pgrep.
#   Notice that the regex used by pgrep matches the start of a process name.
#   We wait up to 5 seconds for the process to start, else fail.
#   This is an imperfect test, and if the docker-run.sh exits before a process is found
#   it may lead to race conditions.
#
# If the test fails we break the docker image build.
RUN set -x; \
    /opt/libra/bin/docker-run.sh &  \
    PIDS=$( pgrep -f '^/usr/bin/python3 /usr/local/bin/gunicorn' | head -n 1 ); \
    COUNTER=5; \
    while [ -z $PIDS ] && [ $COUNTER -gt 0 ]; do \
        sleep 1; \
        PIDS=$( pgrep -f '^/usr/bin/python3 /usr/local/bin/gunicorn' | head -n 1 ); \
        COUNTER=$(( $COUNTER - 1 )); \
    done; \
    if [ -z $PIDS ]; then exit 1; else kill -9 $PIDS || true; fi

FROM pre-test as prod
# Mint proxy listening address
EXPOSE 8000

# Define CFG_SEED, AC_HOST and AC_PORT environment variables when running
CMD /opt/libra/bin/docker-run.sh

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
