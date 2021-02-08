FROM debian:buster-20210111@sha256:b16f66714660c4b3ea14d273ad8c35079b81b35d65d1e206072d226c7ff78299 AS setup_ci

RUN mkdir /diem
COPY rust-toolchain /diem/rust-toolchain
COPY cargo-toolchain /diem/cargo-toolchain
COPY scripts/dev_setup.sh /diem/scripts/dev_setup.sh

# Batch mode and all operations tooling
RUN /diem/scripts/dev_setup.sh -t -o -b -p -y && apt-get clean && rm -rf /var/lib/apt/lists/*
ENV PATH "/root/.cargo/bin:/root/bin/:$PATH"

# Since proper use of sccache demands the exact same path for source and crates, configure it so.
# Paths that every use can choose to setup.
RUN mkdir -p /opt/cargo/
RUN mkdir -p /opt/git/

# Since github actions may not run as root in the docker image, and it's behavior
# is undocument, and all request for information about any invarients are not responded to....
# So we may symlink this locations to whatever home directory gha creates, currently /github/home/
RUN chmod -R o+rwx /root/bin
RUN chmod -R o+rwx /root/.dotnet
RUN chmod -R o+rwx /root/.rustup
RUN chmod -R o+rwx /root/.cargo
RUN chmod -R o+rwx /root/.local
RUN chmod -R o+rwx /root/.nuget

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
    && [ -x "$(command -v "$HOME/.dotnet/tools/boogie")" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/rust-toolchain )" ] \
    && [ -x "$(xargs rustup which cargo --toolchain < /diem/cargo-toolchain)" ]

# should be a no-op
# sccache builds fine, but is not executable ??? in alpine, ends up being recompiled.  Wierd.
RUN /diem/scripts/dev_setup.sh -b -p

FROM setup_ci as build_environment
