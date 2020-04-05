### Production Image ###
FROM libra_e2e:latest as validator_with_config
COPY docker/validator-dynamic/docker-run-dynamic.sh /
COPY docker/validator-dynamic/docker-run-dynamic-fullnode.sh /

CMD /docker-run-dynamic.sh

ARG BUILD_DATE
ARG GIT_REV
ARG GIT_UPSTREAM

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.build-date=$BUILD_DATE
LABEL org.label-schema.vcs-ref=$GIT_REV
LABEL vcs-upstream=$GIT_UPSTREAM
