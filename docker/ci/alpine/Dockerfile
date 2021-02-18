# while using circle we'll use circle's base image.
FROM alpine:3.13.2@sha256:a75afd8b57e7f34e4dad8d65e2c7ba2e1975c795ce1ee22fa34f8cf46f96a3be AS setup_ci_alpine

WORKDIR /diem
COPY rust-toolchain /diem/rust-toolchain
COPY cargo-toolchain /diem/cargo-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

RUN apk add bash=5.1.0-r0 --no-cache

# Batch mode and all operations tooling
RUN scripts/dev_setup.sh -t -o -y -b -p
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

FROM setup_ci_alpine as tested_ci_alpine

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
    && [ -x "$(command -v "$HOME/.dotnet/tools/boogie")" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/rust-toolchain )" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/cargo-toolchain)" ]

# should be a no-op, but since sccache failes to execute, sccache is rebuilt.
#RUN scripts/dev_setup.sh -b -p

FROM setup_ci_alpine as build_environment_alpine
