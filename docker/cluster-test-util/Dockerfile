# Please only make changes to this Dockerfile which are backward compatible. Avoid breaking changes like removing yum packages
# Since we only use the "latest" version of this image, older jobs might still be using the "latest" image
FROM amazonlinux:2

RUN yum -y update && \
    yum install -y git perf procps aws-cli iproute iproute-tc iptables iputils && \
    yum clean all && \
    rm -rf /var/cache/yum && \
    git clone --depth 1 https://github.com/brendangregg/FlameGraph /usr/local/etc/FlameGraph
