#while using circle we'll use circle's base image.
FROM cimg/base:2020.01 AS setup_ci

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
COPY cargo-toolchain /libra/cargo-toolchain
COPY scripts/dev_setup.sh /libra/scripts/dev_setup.sh

#Batch mode and all operations tooling
RUN scripts/dev_setup.sh -o -b -p && sudo apt-get clean
ENV PATH "/home/circleci/.cargo/bin:$PATH"

FROM setup_ci as tested_ci

#Compile a small rust tool?  But we already have in dev_setup (sccache/grcov)...?
#Test that all commands we need are installed and on the PATH
RUN [ -x "$(command -v hadolint)" ] \
    && [ -x "$(command -v vault)" ] \
    && [ -x "$(command -v terraform)" ] \
    && [ -x "$(command -v kubectl)" ] \
    && [ -x "$(command -v rustup)" ] \
    && [ -x "$(command -v cargo)" ] \
    && [ -x "$(command -v sccache)" ] \
    && [ -x "$(command -v grcov)" ] \
    && [ -x "$(command -v helm)" ] \
    && [ -x "$(command -v aws)" ] \
    && [ -x "$(rustup which cargo --toolchain `cat /libra/rust-toolchain`)" ] \
    && [ -x "$(rustup which cargo --toolchain `cat /libra/cargo-toolchain`)" ]

#should be a no-op
# sccache builds fine, but is not executable ??? in alpine, ends up being recompiled.  Wierd.
RUN scripts/dev_setup.sh -b -p

FROM setup_ci as build_environment
