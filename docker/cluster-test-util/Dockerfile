# Please only make changes to this Dockerfile which are backward compatible. Avoid breaking changes like removing yum packages
# Since we only use the "latest" version of this image, older jobs might still be using the "latest" image
FROM amazonlinux:2.0.20200722.0@sha256:1481659e18042055e174f9f5e61998bf7ab57f8c9432f3c9412a56d116cc0c68

RUN yum -y update && \
    yum install -y git perf procps aws-cli iproute iproute-tc iptables iputils && \
    yum clean all && \
    rm -rf /var/cache/yum && \
    git clone --depth 1 https://github.com/brendangregg/FlameGraph /usr/local/etc/FlameGraph
