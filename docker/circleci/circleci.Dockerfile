FROM circleci/rust:buster

RUN sudo sh -c 'echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/backports.list' && \
        sudo apt-get update && \
        sudo apt-get install -y protobuf-compiler/buster cmake curl && \
        sudo apt-get clean && \
        sudo rm -rf /var/lib/apt/lists/* && \
        cargo install sccache

RUN cd /tmp && \
        git clone https://github.com/libra/libra && \
        cd /tmp/libra && \
        rustup component add clippy rustfmt && \
        cargo fetch

CMD ["/bin/sh"]
