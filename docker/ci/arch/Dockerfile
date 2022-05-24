# while using circle we'll use circle's base image.
FROM archlinux:base-devel-20220522.0.57327@sha256:c2646b067fac25f12a0c31574c5f337834fb06fb80d318ac71607d3ec6a9a31d AS setup_ci_arch

WORKDIR /diem
COPY rust-toolchain /diem/rust-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

ENV HOME "/github/home"
#Needed for sccache to function, and to work around home dir being blatted.
ENV CARGO_HOME "/opt/cargo"
ENV RUSTUP_HOME "/opt/rustup"

# WORKAROUND for glibc 2.33 and old Docker
# See https://github.com/actions/virtual-environments/issues/2658
# Thanks to https://github.com/lxqt/lxqt-panel/pull/1562
RUN pacman -Sy archlinux-keyring --noconfirm && \
    rm -rf /etc/pacman.d/gnupg/* && pacman-key --init && pacman-key --populate archlinux && \
    patched_glibc=glibc-linux4-2.33-4-x86_64.pkg.tar.zst && \
    curl -LO "https://repo.archlinuxcn.org/x86_64/$patched_glibc" && \
    bsdtar -C / -xvf "$patched_glibc" && \
    mkdir -p /github/home && \
    mkdir -p /opt/cargo/ && \
    mkdir -p /opt/git/ && \
    /diem/scripts/dev_setup.sh -t -o -b -p -y -n && \
    pacman -Scc --noconfirm

ENV DOTNET_ROOT "/opt/dotnet"
ENV Z3_EXE "/opt/bin/z3"
ENV CVC5_EXE "/opt/bin/cvc5"
ENV BOOGIE_EXE "/opt/dotnet/tools/boogie"
ENV PATH "/opt/cargo/bin:/usr/lib/golang/bin:/opt/bin:${DOTNET_ROOT}:${DOTNET_ROOT}/tools:$PATH"


FROM setup_ci_arch as tested_ci_arch

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
    && [ -x "$(set -x; command -v xsltproc)" ]
# These should eventually be installed and tested, but since we don't automate on arch, low pri.
# && [ -x "$(command -v javac)" ] \
# && [ -x "$(command -v clang)" ] \
# && [ -x "$(command -v python3)" ] \
# && [ -x "$(command -v go)" ] \
# && [ -x "$(command -v npm)" ]

# should be a no-op
RUN scripts/dev_setup.sh -t -o -y -b -p

FROM setup_ci_arch as build_environment_arch
