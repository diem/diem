FROM debian:buster-20211201@sha256:5b57f8c365c40fde437d53b953c436995525be7c481eb0128b1cbf3b49b0df18 AS setup_ci

RUN mkdir /diem
COPY rust-toolchain /diem/rust-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

#this is the default home on docker images in gha, until it's not?
ENV HOME "/github/home"
#Needed for sccache to function, and to work around home dir being blatted.
ENV CARGO_HOME "/opt/cargo"
ENV RUSTUP_HOME "/opt/rustup"

# Batch mode and all operations tooling
RUN mkdir -p /github/home && \
    mkdir -p /opt/cargo/ && \
    mkdir -p /opt/git/ && \
    /diem/scripts/dev_setup.sh -t -o -b -p -y -s -n -a && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV DOTNET_ROOT "/opt/dotnet"
ENV Z3_EXE "/opt/bin/z3"
ENV CVC5_EXE "/opt/bin/cvc5"
ENV BOOGIE_EXE "/opt/dotnet/tools/boogie"
ENV PATH "/opt/cargo/bin:/usr/lib/golang/bin:/opt/bin:${DOTNET_ROOT}:${DOTNET_ROOT}/tools:$PATH"

FROM setup_ci as tested_ci

# Compile a small rust tool?  But we already have in dev_setup (sccache/grcov)...?
# Test that all commands we need are installed and on the PATH
RUN [ -x "$(set -x; command -v shellcheck)" ] \
    && [ -x "$(set -x; command -v hadolint)" ] \
    && [ -x "$(set -x; command -v vault)" ] \
    && [ -x "$(set -x; command -v terraform)" ] \
    && [ -x "$(set -x; command -v kubectl)" ] \
    && [ -x "$(set -x; command -v rustup)" ] \
    && [ -x "$(set -x; command -v cargo)" ] \
    && [ -x "$(set -x; command -v cargo-guppy)" ] \
    && [ -x "$(set -x; command -v sccache)" ] \
    && [ -x "$(set -x; command -v grcov)" ] \
    && [ -x "$(set -x; command -v helm)" ] \
    && [ -x "$(set -x; command -v aws)" ] \
    && [ -x "$(set -x; command -v z3)" ] \
    && [ -x "$(set -x; command -v "$BOOGIE_EXE")" ] \
    && [ -x "$(set -x; xargs rustup which cargo --toolchain < /diem/rust-toolchain )" ] \
    && [ -x "$(set -x; command -v tidy)" ] \
    && [ -x "$(set -x; command -v xsltproc)" ] \
    && [ -x "$(set -x; command -v javac)" ] \
    && [ -x "$(set -x; command -v clang)" ] \
    && [ -x "$(set -x; command -v python3)" ] \
    && [ -x "$(set -x; command -v go)" ] \
    && [ -x "$(set -x; command -v npm)" ] \
    && [ -x "$(set -x; command -v deno)" ]

# should be a no-op
# sccache builds fine, but is not executable ??? in alpine, ends up being recompiled.  Wierd.
RUN /diem/scripts/dev_setup.sh -t -o -y -b -p -s -a

FROM setup_ci as build_environment
