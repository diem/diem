FROM circleci/rust:buster

RUN cargo install sccache

RUN cd /tmp && \
        git clone https://github.com/libra/libra && \
        cd /tmp/libra && \
        rustup component add clippy rustfmt && \
        cargo fetch

CMD ["/bin/sh"]
