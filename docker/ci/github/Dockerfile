FROM debian:buster-20210311@sha256:9d4ab94af82b2567c272c7f47fa1204cd9b40914704213f1c257c44042f82aac AS setup_ci

RUN mkdir /diem
COPY rust-toolchain /diem/rust-toolchain
COPY cargo-toolchain /diem/cargo-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

#this is the default home on docker images in gha, until it's not?
ENV HOME "/github/home"
#Needed for sccache to function
ENV CARGO_HOME "/opt/cargo/"

# Batch mode and all operations tooling
RUN mkdir -p /github/home \
    && mkdir -p /opt/cargo/ \
    && mkdir -p /opt/git/ \
    && /diem/scripts/dev_setup.sh -t -o -b -p -y -s\
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV PATH "/opt/cargo/bin:/usr/lib/golang/bin:/github/home/bin:$PATH"
ENV DOTNET_ROOT "/github/home/.dotnet"
ENV Z3_EXE "/github/home/bin/z3"
ENV BOOGIE_EXE "/github/home/.dotnet/tools/boogie"

FROM setup_ci as tested_ci

# Compile a small rust tool?  But we already have in dev_setup (sccache/grcov)...?
# Test that all commands we need are installed and on the PATH
RUN [ -x "$(command -v shellcheck)" ] \
    && [ -x "$(command -v hadolint)" ] \
    && [ -x "$(command -v vault)" ] \
    && [ -x "$(command -v terraform)" ] \
    && [ -x "$(command -v kubectl)" ] \
    && [ -x "$(command -v rustup)" ] \
    && [ -x "$(command -v cargo)" ] \
    && [ -x "$(command -v sccache)" ] \
    && [ -x "$(command -v grcov)" ] \
    && [ -x "$(command -v helm)" ] \
    && [ -x "$(command -v aws)" ] \
    && [ -x "$(command -v z3)" ] \
    && [ -x "$(command -v javac)" ] \
    && [ -x "$(command -v clang)" ] \
    && [ -x "$(command -v python3)" ] \
    && [ -x "$(command -v go)" ] \
    && [ -x "$(command -v npm)" ] \
    && [ -x "$(command -v "$HOME/.dotnet/tools/boogie")" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/rust-toolchain )" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/cargo-toolchain)" ]

# should be a no-op
# sccache builds fine, but is not executable ??? in alpine, ends up being recompiled.  Wierd.
RUN /diem/scripts/dev_setup.sh -b -p -s

FROM setup_ci as build_environment
