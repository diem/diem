FROM debian:buster AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

RUN apt-get update && apt-get install -y cmake curl clang git

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
RUN rustup install $(cat rust-toolchain)

FROM toolchain AS config_builder

COPY . /libra

RUN cargo build --release -p libra-node -p client -p config-builder && cd target/release && rm -r build deps incremental

### Production Image ###
FROM libra_e2e:latest as validator_with_config
COPY --from=config_builder /libra/target/release/config-builder /opt/libra/bin
COPY docker/validator-dynamic/docker-run-dynamic.sh /
COPY docker/validator-dynamic/docker-run-dynamic-fullnode.sh /
COPY docker/logstash-config.conf /

CMD /docker-run-dynamic.sh

RUN echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list \
    && apt-get update && apt-get install -y openjdk-11-jdk wget gnupg
RUN wget -qO - https://artifacts.elastic.co/GPG-KEY-elasticsearch | apt-key add -
RUN echo "deb https://artifacts.elastic.co/packages/7.x/apt stable main" > /etc/apt/sources.list.d/elastic-7.x.list \
    && export JAVA_HOME=$(which java) \
    && apt-get update && apt-get install logstash
RUN /usr/share/logstash/bin/logstash-plugin install logstash-output-amazon_es
RUN /usr/share/logstash/bin/logstash -f /home/logstash-config -r > /dev/null 2>&1 &

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
