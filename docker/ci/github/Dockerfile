FROM debian:buster-20201209@sha256:5b5fa7e155b1f19dffb996ea64e55520b80d5bd7a8fdb5aed1acabd217e9ed59 AS setup_ci
#FROM debian:buster-20201117-slim@sha256:21ce4b82ed3425e757b1ac98ff9cbeb83540034c0cbc32766a59c3eca9e0daf8 AS setup_ci


RUN mkdir /diem
COPY rust-toolchain /diem/rust-toolchain
COPY cargo-toolchain /diem/cargo-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

#Batch mode and all operations tooling
RUN /diem/scripts/dev_setup.sh -t -o -b -p -y
RUN apt-get clean && rm -rf /var/lib/apt/lists/*
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

#Since proper use of sccache demands the exact same path for source and crates, configure it so.
#Paths that every use can choose to setup.
RUN mkdir -p /opt/cargo/
RUN mkdir -p /opt/git/

FROM setup_ci as tested_ci

#Compile a small rust tool?  But we already have in dev_setup (sccache/grcov)...?
#Test that all commands we need are installed and on the PATH
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
    && [ -x "$(command -v "$HOME/.dotnet/tools/boogie")" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/rust-toolchain )" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/cargo-toolchain)" ]

#should be a no-op
# sccache builds fine, but is not executable ??? in alpine, ends up being recompiled.  Wierd.
RUN /diem/scripts/dev_setup.sh -b -p

FROM setup_ci as build_environment
