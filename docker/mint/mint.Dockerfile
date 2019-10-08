### Production Image ###
FROM libra_base as builder
FROM debian:stretch

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
