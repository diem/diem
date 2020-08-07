#while using circle we'll use circle's base image.
FROM centos:latest AS setup_ci_centos

WORKDIR /libra
COPY rust-toolchain /libra/rust-toolchain
COPY cargo-toolchain /libra/cargo-toolchain
COPY scripts/dev_setup.sh /libra/scripts/dev_setup.sh

#Batch mode and all operations tooling
RUN scripts/dev_setup.sh -o -b -p && yum clean all
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

FROM setup_ci_centos as tested_ci_centos

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
RUN scripts/dev_setup.sh -b -p

FROM setup_ci_centos as build_environment_centos
