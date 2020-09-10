#while using circle we'll use circle's base image.
FROM archlinux:20200705@sha256:ce7ccce2c458b875f66a0e33d492ccc2f5495dbf21a7fc9acd261f8f980d4fd5 AS setup_ci_arch

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
COPY cargo-toolchain /libra/cargo-toolchain
COPY scripts/dev_setup.sh /libra/scripts/dev_setup.sh

#Batch mode and all operations tooling
RUN scripts/dev_setup.sh -o -b -p && pacman -Scc --noconfirm
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

FROM setup_ci_arch as tested_ci_arch

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
    && [ -x "$(xargs rustup which cargo --toolchain < /libra/rust-toolchain )" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /libra/cargo-toolchain)" ]

#should be a no-op
RUN scripts/dev_setup.sh -b -p

FROM setup_ci_arch as build_environment_arch
