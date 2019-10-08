### Base Builder Image ##
FROM libra_base as builder

### Production Image ###
FROM debian:stretch

RUN mkdir -p /opt/libra/bin /opt/libra/etc

COPY --from=builder /libra/target/release/ruben /opt/libra/bin
COPY docker/bench/bench_init.sh /opt/libra/bin/
RUN chmod +x /opt/libra/bin/bench_init.sh

# Metrics
EXPOSE 9101

# Define MINT_KEY, AC_HOST and AC_DEBUG environment variables when running
ENTRYPOINT ["/opt/libra/bin/bench_init.sh"]


ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
