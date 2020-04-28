
### Build layer
ARG TOOLCHAIN_TAG
FROM toolchain_rust:$TOOLCHAIN_TAG AS builder

WORKDIR /libra
COPY . /libra

RUN cargo build --release -p libra-node -p cli -p config-builder -p safety-rules && cd target/release && rm -r build deps incremental

### Production Image ###
FROM debian:buster AS pre-test

# TODO: Unsure which of these are needed exactly for client
RUN apt-get update \
    && apt-get install -y \
        iproute2=4.20.0-2 \
        nano=3.2-3 \
        net-tools=1.60+git20180626.aebd88e-1 \
        netcat=1.10-41.1 \
        python3-pip=18.1-5 \
        tcpdump=4.9.3-1~deb10u1 \
    && apt-get clean \
    && rm -r /var/lib/apt/lists/*

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
RUN apt-get update \
    && apt-get install -y procps=2:3.3.15-2 \
    && apt-get clean \
    && rm -r /var/lib/apt/lists/*

# TEST docker-run.sh
#   GIVEN necessary environment arguments to run docker-run.sh
ARG AC_PORT="8080"
ARG AC_HOST="172.18.0.13"
ARG LOG_LEVEL="info"

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
