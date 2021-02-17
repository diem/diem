# while using circle we'll use circle's base image.
FROM archlinux:base-devel-20210214.0.15477@sha256:225d6241fc4bf9a61b849a7cac9aab8e59e89da889ef5d569ea96b9d9f7e503c AS setup_ci_arch

WORKDIR /diem
COPY rust-toolchain /diem/rust-toolchain
COPY cargo-toolchain /diem/cargo-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

# Batch mode and all operations tooling
RUN scripts/dev_setup.sh -t -o -y -b -p && pacman -Scc --noconfirm
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

FROM setup_ci_arch as tested_ci_arch

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

# should be a no-op
RUN scripts/dev_setup.sh -b -p

FROM setup_ci_arch as build_environment_arch
