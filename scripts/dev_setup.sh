#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
# This script sets up the environment for the Diem build by installing necessary dependencies.
#
# Usage ./dev_setup.sh <options>
#   v - verbose, print all statements

# Assumptions for nix systems:
# 1 The running user is the user who will execute the builds.
# 2 .profile will be used to configure the shell
# 3 ${HOME}/bin/, or ${INSTALL_DIR} is expected to be on the path - hashicorp tools/hadolint/etc.  will be installed there on linux systems.

# fast fail.
set -eo pipefail

SHELLCHECK_VERSION=0.7.1
HADOLINT_VERSION=1.17.4
SCCACHE_VERSION=0.2.16-alpha.0
#If installing sccache from a git repp set url@revision.
SCCACHE_GIT='https://github.com/diem/sccache.git@ef50d87a58260c30767520045e242ccdbdb965af'
GRCOV_VERSION=0.8.2
GUPPY_GIT='https://github.com/facebookincubator/cargo-guppy@39ec940f36b0a0df96a330243d127cbe2db9f919'
KUBECTL_VERSION=1.18.6
TERRAFORM_VERSION=0.12.26
HELM_VERSION=3.2.4
VAULT_VERSION=1.5.0
Z3_VERSION=4.8.12
CVC4_VERSION=aac53f51
CVC5_VERSION=0.0.3
DOTNET_VERSION=5.0
BOOGIE_VERSION=2.9.6
PYRE_CHECK_VERSION=0.0.59
NUMPY_VERSION=1.20.1
ALLURE_VERSION=2.15.pr1135

SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

function usage {
  echo "Usage:"
  echo "Installs or updates necessary dev tools for diem/diem."
  echo "-b batch mode, no user interactions and miminal output"
  echo "-p update ${HOME}/.profile"
  echo "-t install build tools"
  echo "-o install operations tooling as well: helm, terraform, hadolint, yamllint, vault, docker, kubectl, python3"
  echo "-y installs or updates Move prover tools: z3, cvc4, cvc5, dotnet, boogie"
  echo "-s installs or updates requirements to test code-generation for Move SDKs"
  echo "-a install tools for build and test api"
  echo "-v verbose mode"
  echo "-i installs an individual tool by name"
  echo "-n will target the /opt/ dir rather than the $HOME dir.  /opt/bin/, /opt/rustup/, and /opt/dotnet/ rather than $HOME/bin/, $HOME/.rustup/, and $HOME/.dotnet/"
  echo "If no toolchain component is selected with -t, -o, -y, or -p, the behavior is as if -t had been provided."
  echo "This command must be called from the root folder of the Diem project."
}

function add_to_profile {
  eval "$1"
  FOUND=$(grep -c "$1" < "${HOME}/.profile" || true)  # grep error return would kill the script.
  if [ "$FOUND" == "0" ]; then
    echo "$1" >> "${HOME}"/.profile
  fi
}


# It is important to keep all path updates together to allow this script to work well when run in github actions
# inside of a docker image created using this script.   GHA wipes the home directory via docker mount options, so
# this profile needs built and sourced on every execution of a job using the docker image.   See the .github/actions/build-setup
# action in this repo, as well as docker/ci/github/Dockerfile.
function update_path_and_profile {
  touch "${HOME}"/.profile

  DOTNET_ROOT="$HOME/.dotnet"
  BIN_DIR="$HOME/bin"
  C_HOME="${HOME}/.cargo"
  if [[ "$OPT_DIR" == "true" ]]; then
    DOTNET_ROOT="/opt/dotnet"
    BIN_DIR="/opt/bin"
    C_HOME="/opt/cargo"
  fi

  mkdir -p "${BIN_DIR}"
  if [ -n "$CARGO_HOME" ]; then
    add_to_profile "export CARGO_HOME=\"${CARGO_HOME}\""
    add_to_profile "export PATH=\"${BIN_DIR}:${CARGO_HOME}/bin:\$PATH\""
  else
    add_to_profile "export PATH=\"${BIN_DIR}:${C_HOME}/bin:\$PATH\""
  fi
  if [[ "$INSTALL_PROVER" == "true" ]]; then
    add_to_profile "export DOTNET_ROOT=\"${DOTNET_ROOT}\""
    add_to_profile "export PATH=\"${DOTNET_ROOT}/tools:\$PATH\""
    add_to_profile "export Z3_EXE=\"${BIN_DIR}/z3\""
    add_to_profile "export CVC4_EXE=\"${BIN_DIR}/cvc4\""
    add_to_profile "export CVC5_EXE=\"${BIN_DIR}/cvc5\""
    add_to_profile "export BOOGIE_EXE=\"${DOTNET_ROOT}/tools/boogie\""
  fi
  if [[ "$INSTALL_CODEGEN" == "true" ]] && [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    if [[ -n "${GOBIN}" ]]; then
      add_to_profile "export PATH=\$PATH:/usr/lib/golang/bin"
    else
      add_to_profile "export PATH=\$PATH:$GOBIN"
    fi
  fi
}

function install_build_essentials {
  PACKAGE_MANAGER=$1
  #Differently named packages for pkg-config
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    install_pkg build-essential "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
    install_pkg base-devel "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "apk" ]]; then
    install_pkg alpine-sdk "$PACKAGE_MANAGER"
    install_pkg coreutils "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "yum" ]] || [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
    install_pkg gcc "$PACKAGE_MANAGER"
    install_pkg gcc-c++ "$PACKAGE_MANAGER"
    install_pkg make "$PACKAGE_MANAGER"
  fi
  #if [[ "$PACKAGE_MANAGER" == "brew" ]]; then
  #  install_pkg pkgconfig "$PACKAGE_MANAGER"
  #fi
}

function install_rustup {
  echo installing rust.
  BATCH_MODE=$1
  if [[ "$OPT_DIR" == "true" ]]; then
     export RUSTUP_HOME=/opt/rustup/
     mkdir -p "$RUSTUP_HOME" || true
     export CARGO_HOME=/opt/cargo/
     mkdir -p "$CARGO_HOME" || true
  fi

  # Install Rust
  if [[ "${BATCH_MODE}" == "false" ]]; then
    echo "Installing Rust......"
  fi
  VERSION="$(rustup --version || true)"
  if [ -n "$VERSION" ]; then
	  if [[ "${BATCH_MODE}" == "false" ]]; then
      echo "Rustup is already installed, version: $VERSION"
    fi
  else
	  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
    if [[ -n "${CARGO_HOME}" ]]; then
      PATH="${CARGO_HOME}/bin:${PATH}"
    else
      PATH="${HOME}/.cargo/bin:${PATH}"
    fi
  fi
}

function install_hadolint {
  if ! command -v hadolint &> /dev/null; then
    export HADOLINT=${INSTALL_DIR}/hadolint
    curl -sL -o "$HADOLINT" "https://github.com/hadolint/hadolint/releases/download/v${HADOLINT_VERSION}/hadolint-$(uname -s)-$(uname -m)" && chmod 700 "$HADOLINT"
  fi
  hadolint -v
}

function install_vault {
  VERSION=$("${INSTALL_DIR}"/vault --version || true)
  if [[ "$VERSION" != "Vault v${VAULT_VERSION}" ]]; then
    MACHINE=$(uname -m);
    if [[ $MACHINE == "x86_64" ]]; then
      MACHINE="amd64"
    fi
    TMPFILE=$(mktemp)
    curl -sL -o "$TMPFILE" "https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_$(uname -s | tr '[:upper:]' '[:lower:]')_${MACHINE}.zip"
    unzip -qq -d "$INSTALL_DIR" "$TMPFILE"
    rm "$TMPFILE"
    chmod +x "${INSTALL_DIR}"/vault
  fi
  "${INSTALL_DIR}"/vault --version
}

function install_helm {
  if ! command -v helm &> /dev/null; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg helm brew
    else
      MACHINE=$(uname -m);
      if [[ $MACHINE == "x86_64" ]]; then
        MACHINE="amd64"
      fi
      TMPFILE=$(mktemp)
      rm "$TMPFILE"
      mkdir -p "$TMPFILE"/
      curl -sL -o "$TMPFILE"/out.tar.gz "https://get.helm.sh/helm-v${HELM_VERSION}-$(uname -s | tr '[:upper:]' '[:lower:]')-${MACHINE}.tar.gz"
      tar -zxvf "$TMPFILE"/out.tar.gz -C "$TMPFILE"/
      cp "${TMPFILE}/$(uname -s | tr '[:upper:]' '[:lower:]')-${MACHINE}/helm" "${INSTALL_DIR}/helm"
      rm -rf "$TMPFILE"
      chmod +x "${INSTALL_DIR}"/helm
    fi
  fi
}

function install_terraform {
  VERSION=$(terraform --version | head -1 || true)
  if [[ "$VERSION" != "Terraform v${TERRAFORM_VERSION}" ]]; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg tfenv brew
      tfenv install ${TERRAFORM_VERSION}
      tfenv use ${TERRAFORM_VERSION}
    else
      MACHINE=$(uname -m);
      if [[ $MACHINE == "x86_64" ]]; then
        MACHINE="amd64"
      fi
      TMPFILE=$(mktemp)
      curl -sL -o "$TMPFILE" "https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_$(uname -s | tr '[:upper:]' '[:lower:]')_${MACHINE}.zip"
      unzip -qq -d "${INSTALL_DIR}" "$TMPFILE"
      rm "$TMPFILE"
      chmod +x "${INSTALL_DIR}"/terraform
      terraform --version
    fi
  fi
}

function install_kubectl {
  VERSION=$(kubectl version client --short=true | head -1 || true)
  if [[ "$VERSION" != "Client Version: v${KUBECTL_VERSION}" ]]; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg kubectl brew
    else
      MACHINE=$(uname -m);
      if [[ $MACHINE == "x86_64" ]]; then
        MACHINE="amd64"
      fi
      curl -sL -o "${INSTALL_DIR}"/kubectl "https://storage.googleapis.com/kubernetes-release/release/v${KUBECTL_VERSION}/bin/$(uname -s | tr '[:upper:]' '[:lower:]')/${MACHINE}/kubectl"
      chmod +x "${INSTALL_DIR}"/kubectl
    fi
  fi
  kubectl version client --short=true | head -1 || true
}

function install_awscli {
  PACKAGE_MANAGER=$1
  if ! command -v aws &> /dev/null; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg awscli brew
    elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
      apk add --no-cache python3 py3-pip \
      && pip3 install --upgrade pip \
      && pip3 install awscli
    else
      MACHINE=$(uname -m);
      TMPFILE=$(mktemp)
      rm "$TMPFILE"
      mkdir -p "$TMPFILE"/work/
      curl -sL -o "$TMPFILE"/aws.zip  "https://awscli.amazonaws.com/awscli-exe-$(uname -s | tr '[:upper:]' '[:lower:]')-${MACHINE}.zip"
      unzip -qq -d "$TMPFILE"/work/ "$TMPFILE"/aws.zip
      TARGET_DIR="${HOME}"/.local/
      if [[ "$OPT_DIR" == "true" ]]; then
         TARGET_DIR="/opt/aws/"
      fi
      mkdir -p "${TARGET_DIR}"
      "$TMPFILE"/work/aws/install -i "${TARGET_DIR}" -b "${INSTALL_DIR}"
      "${INSTALL_DIR}"aws --version
    fi
  fi
}

function install_pkg {
  package=$1
  PACKAGE_MANAGER=$2
  PRE_COMMAND=()
  if [ "$(whoami)" != 'root' ]; then
    PRE_COMMAND=(sudo)
  fi
  if command -v "$package" &>/dev/null; then
    echo "$package is already installed"
  else
    echo "Installing ${package}."
    if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
      "${PRE_COMMAND[@]}" yum install "${package}" -y
    elif [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      "${PRE_COMMAND[@]}" apt-get install "${package}" --no-install-recommends -y
      echo apt-get install result code: $?
    elif [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
      "${PRE_COMMAND[@]}" pacman -Syu "$package" --noconfirm
    elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
      apk --update add --no-cache "${package}"
    elif [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
      dnf install "$package"
    elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
      brew install "$package"
    fi
  fi
}

function install_pkg_config {
  PACKAGE_MANAGER=$1
  #Differently named packages for pkg-config
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]] || [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
    install_pkg pkg-config "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
    install_pkg pkgconf "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "brew" ]] || [[ "$PACKAGE_MANAGER" == "apk" ]] || [[ "$PACKAGE_MANAGER" == "yum" ]]; then
    install_pkg pkgconfig "$PACKAGE_MANAGER"
  fi
}

function install_shellcheck {
  if ! command -v shellcheck &> /dev/null; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg shellcheck brew
    else
      install_pkg xz "$PACKAGE_MANAGER"
      MACHINE=$(uname -m);
      TMPFILE=$(mktemp)
      rm "$TMPFILE"
      mkdir -p "$TMPFILE"/
      curl -sL -o "$TMPFILE"/out.xz "https://github.com/koalaman/shellcheck/releases/download/v${SHELLCHECK_VERSION}/shellcheck-v${SHELLCHECK_VERSION}.$(uname -s | tr '[:upper:]' '[:lower:]').${MACHINE}.tar.xz"
      tar -xf "$TMPFILE"/out.xz -C "$TMPFILE"/
      cp "${TMPFILE}/shellcheck-v${SHELLCHECK_VERSION}/shellcheck" "${INSTALL_DIR}/shellcheck"
      rm -rf "$TMPFILE"
      chmod +x "${INSTALL_DIR}"/shellcheck
    fi
  fi
}

function install_openssl_dev {
  PACKAGE_MANAGER=$1
  #Differently named packages for openssl dev
  if [[ "$PACKAGE_MANAGER" == "apk" ]]; then
    install_pkg openssl-dev "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    install_pkg libssl-dev "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "yum" ]] || [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
    install_pkg openssl-devel "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "pacman" ]] || [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    install_pkg openssl "$PACKAGE_MANAGER"
  fi
}

function install_lcov {
  PACKAGE_MANAGER=$1
  #Differently named packages for lcov with different sources.
  if [[ "$PACKAGE_MANAGER" == "apk" ]]; then
    apk --update add --no-cache  -X http://dl-cdn.alpinelinux.org/alpine/edge/testing lcov
  fi
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]] || [[ "$PACKAGE_MANAGER" == "yum" ]] || [[ "$PACKAGE_MANAGER" == "dnf" ]] || [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    install_pkg lcov "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
    echo nope no lcov for you.
    echo You can try installing yourself with:
    echo install_pkg git "$PACKAGE_MANAGER"
    echo cd lcov;
    echo git clone https://aur.archlinux.org/lcov.git
    echo makepkg -si --noconfirm
  fi
}

function install_tidy {
  PACKAGE_MANAGER=$1
  #Differently named packages for tidy
  if [[ "$PACKAGE_MANAGER" == "apk" ]]; then
    apk --update add --no-cache  -X http://dl-cdn.alpinelinux.org/alpine/edge/testing tidyhtml
  else
    install_pkg tidy "$PACKAGE_MANAGER"
  fi
}

function install_gcc_powerpc_linux_gnu {
  PACKAGE_MANAGER=$1
  #Differently named packages for gcc-powerpc-linux-gnu
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]] || [[ "$PACKAGE_MANAGER" == "yum" ]]; then
    install_pkg gcc-powerpc-linux-gnu "$PACKAGE_MANAGER"
  fi
  #if [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
  #  install_pkg powerpc-linux-gnu-gcc "$PACKAGE_MANAGER"
  #fi
  #if [[ "$PACKAGE_MANAGER" == "apk" ]] || [[ "$PACKAGE_MANAGER" == "brew" ]]; then
  #  TODO
  #fi
}

function install_toolchain {
  version=$1
  FOUND=$(rustup show | grep -c "$version" || true )
  if [[ "$FOUND" == "0" ]]; then
    echo "Installing ${version} of rust toolchain"
    rustup install "$version"
  else
    echo "${version} rust toolchain already installed"
  fi
}

function install_sccache {
  VERSION="$(sccache --version || true)"
  if [[ "$VERSION" != "sccache ""${SCCACHE_VERSION}" ]]; then
    if [[ -n "${SCCACHE_GIT}" ]]; then
      git_repo=$( echo "$SCCACHE_GIT" | cut -d "@" -f 1 );
      git_hash=$( echo "$SCCACHE_GIT" | cut -d "@" -f 2 );
      cargo install sccache --git "$git_repo" --rev "$git_hash" --features s3 --locked
    else
      cargo install sccache --version="${SCCACHE_VERSION}" --features s3 --locked
    fi
  fi
}

function install_cargo_guppy {
  if ! command -v cargo-guppy &> /dev/null; then
    git_repo=$( echo "$GUPPY_GIT" | cut -d "@" -f 1 );
    git_hash=$( echo "$GUPPY_GIT" | cut -d "@" -f 2 );
    cargo install cargo-guppy --git "$git_repo" --rev "$git_hash" --locked
  fi
}

function install_grcov {
  if ! command -v grcov &> /dev/null; then
    cargo install grcov --version="${GRCOV_VERSION}" --locked
  fi
}

function install_dotnet {
  echo "Installing .Net"
  mkdir -p "${DOTNET_INSTALL_DIR}" || true
  if [[ $("${DOTNET_INSTALL_DIR}/dotnet" --list-sdks | grep -c "^${DOTNET_VERSION}" || true) == "0" ]]; then
    if [[ "$(uname)" == "Linux" ]]; then
        # Install various prerequisites for .dotnet. There are known bugs
        # in the dotnet installer to warn even if they are present. We try
        # to install anyway based on the warnings the dotnet installer creates.
        if [ "$PACKAGE_MANAGER" == "apk" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
          install_pkg libintl "$PACKAGE_MANAGER"
          install_pkg libcurl "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "apt-get" ]; then
          install_pkg gettext "$PACKAGE_MANAGER"
          install_pkg zlib1g "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "pacman" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
        fi
    fi
    # Below we need to (a) set TERM variable because the .net installer expects it and it is not set
    # in some environments (b) use bash not sh because the installer uses bash features.
    curl -sSL https://dot.net/v1/dotnet-install.sh \
        | TERM=linux /bin/bash -s -- --channel $DOTNET_VERSION --install-dir "${DOTNET_INSTALL_DIR}" --version latest
  else
    echo Dotnet already installed.
  fi
}

function install_boogie {
  echo "Installing boogie"
  mkdir -p "${DOTNET_INSTALL_DIR}tools/" || true
  if [[ "$("${DOTNET_INSTALL_DIR}dotnet" tool list --tool-path "${DOTNET_INSTALL_DIR}tools/")" =~ .*boogie.*${BOOGIE_VERSION}.* ]]; then
    echo "Boogie $BOOGIE_VERSION already installed"
  else
    "${DOTNET_INSTALL_DIR}dotnet" tool update --tool-path "${DOTNET_INSTALL_DIR}tools/" Boogie --version $BOOGIE_VERSION
  fi
}

function install_z3 {
  echo "Installing Z3"
  if command -v /usr/local/bin/z3 &>/dev/null; then
    echo "z3 already exists at /usr/local/bin/z3"
    echo "but this install will go to ${INSTALL_DIR}/z3."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if command -v "${INSTALL_DIR}z3" &>/dev/null && [[ "$("${INSTALL_DIR}z3" --version || true)" =~ .*${Z3_VERSION}.* ]]; then
     echo "Z3 ${Z3_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-glibc-2.31"
  elif [[ "$(uname)" == "Darwin" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-osx-10.15.7"
  else
    echo "Z3 support not configured for this platform (uname=$(uname))"
    return
  fi
  TMPFILE=$(mktemp)
  rm "$TMPFILE"
  mkdir -p "$TMPFILE"/
  (
    cd "$TMPFILE" || exit
    curl -LOs "https://github.com/Z3Prover/z3/releases/download/z3-$Z3_VERSION/$Z3_PKG.zip"
    unzip -q "$Z3_PKG.zip"
    cp "$Z3_PKG/bin/z3" "${INSTALL_DIR}"
    chmod +x "${INSTALL_DIR}z3"
  )
  rm -rf "$TMPFILE"
}

function install_cvc4 {
  echo "Installing CVC4"
  if command -v /usr/local/bin/cvc4 &>/dev/null; then
    echo "cvc4 already exists at /usr/local/bin/cvc4"
    echo "but this install will go to $${INSTALL_DIR}cvc4."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if command -v "${INSTALL_DIR}cvc4" &>/dev/null && [[ "$("${INSTALL_DIR}cvc4" --version || true)" =~ .*${CVC4_VERSION}.* ]]; then
     echo "CVC4 ${CVC4_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    CVC4_PKG="cvc4-$CVC4_VERSION-x64-ubuntu"
  elif [[ "$(uname)" == "Darwin" ]]; then
    CVC4_PKG="cvc4-$CVC4_VERSION-x64-osx"
  else
    echo "CVC4 support not configured for this platform (uname=$(uname))"
    return
  fi
  TMPFILE=$(mktemp)
  rm "$TMPFILE"
  mkdir -p "$TMPFILE"/
  (
    cd "$TMPFILE" || exit
    curl -LOs "https://cvc4.cs.stanford.edu/downloads/builds/minireleases/$CVC4_PKG.zip"
    unzip -q "$CVC4_PKG.zip"
    cp "$CVC4_PKG/cvc4" "${INSTALL_DIR}"
    chmod +x "${INSTALL_DIR}cvc4"
  )
  rm -rf "$TMPFILE"
}

function install_cvc5 {
  echo "Installing cvc5"
  if command -v /usr/local/bin/cvc5 &>/dev/null; then
    echo "cvc5 already exists at /usr/local/bin/cvc5"
    echo "but this install will go to $${INSTALL_DIR}cvc5."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if command -v "${INSTALL_DIR}cvc5" &>/dev/null && [[ "$("${INSTALL_DIR}cvc5" --version || true)" =~ .*${CVC5_VERSION}.* ]]; then
     echo "cvc5 ${CVC5_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    CVC5_PKG="cvc5-Linux"
  elif [[ "$(uname)" == "Darwin" ]]; then
    CVC5_PKG="cvc5-macOS"
  else
    echo "cvc5 support not configured for this platform (uname=$(uname))"
    return
  fi
  TMPFILE=$(mktemp)
  rm "$TMPFILE"
  mkdir -p "$TMPFILE"/
  (
    cd "$TMPFILE" || exit
    curl -LOs "https://github.com/cvc5/cvc5/releases/download/cvc5-$CVC5_VERSION/$CVC5_PKG"
    cp "$CVC5_PKG" "${INSTALL_DIR}cvc5"
    chmod +x "${INSTALL_DIR}cvc5"
  )
  rm -rf "$TMPFILE"
}

function install_golang {
    if [[ $(go version | grep -c "go1.14" || true) == "0" ]]; then
      if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
        curl -LO https://golang.org/dl/go1.14.15.linux-amd64.tar.gz
        "${PRE_COMMAND[@]}" rm -rf /usr/local/go
        "${PRE_COMMAND[@]}" tar -C /usr/local -xzf go1.14.15.linux-amd64.tar.gz
        "${PRE_COMMAND[@]}" ln -sf /usr/local/go /usr/lib/golang
        rm go1.14.15.linux-amd64.tar.gz
      elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
        apk --update add --no-cache git make musl-dev go
      elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
        failed=$(brew install go || echo "failed")
        if [[ "$failed" == "failed" ]]; then
          brew link --overwrite go
        fi
      else
        install_pkg golang "$PACKAGE_MANAGER"
      fi
    fi
}

function install_deno {
  curl -fsSL https://deno.land/x/install/install.sh | sh
  cp "${HOME}/.deno/bin/deno" "${INSTALL_DIR}"
  chmod +x "${INSTALL_DIR}deno"
}

function install_java {
    if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      "${PRE_COMMAND[@]}" apt-get install -y default-jdk
    elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
      apk --update add --no-cache  -X http://dl-cdn.alpinelinux.org/alpine/edge/community openjdk11
    else
      install_pkg java "$PACKAGE_MANAGER"
    fi
}

function install_allure {
    VERSION="$(allure --version || true)"
    if [[ "$VERSION" != "${ALLURE_VERSION}" ]]; then
      if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
        "${PRE_COMMAND[@]}" apt-get install default-jre -y --no-install-recommends
        export ALLURE=${HOME}/allure_"${ALLURE_VERSION}"-1_all.deb
        curl -sL -o "$ALLURE" "https://github.com/diem/allure2/releases/download/${ALLURE_VERSION}/allure_${ALLURE_VERSION}-1_all.deb"
        "${PRE_COMMAND[@]}" dpkg -i "$ALLURE"
        rm "$ALLURE"
      elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
        apk --update add --no-cache  -X http://dl-cdn.alpinelinux.org/alpine/edge/community openjdk11
      else
        echo No good way to install allure 'install_pkg allure '"$PACKAGE_MANAGER"
      fi
    fi
}

function install_xsltproc {
    if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      install_pkg xsltproc "$PACKAGE_MANAGER"
    else
      install_pkg libxslt "$PACKAGE_MANAGER"
    fi
}

function install_nodejs {
    if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      curl -fsSL https://deb.nodesource.com/setup_14.x | bash -
      apt-get install -y nodejs
    else
      install_pkg nodejs "$PACKAGE_MANAGER"
    fi
    install_pkg npm "$PACKAGE_MANAGER"
}

function install_python3 {
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    install_pkg python3-all-dev "$PACKAGE_MANAGER"
    install_pkg python3-setuptools "$PACKAGE_MANAGER"
    install_pkg python3-pip "$PACKAGE_MANAGER"
  elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
    install_pkg python3-dev "$PACKAGE_MANAGER"
  else
    install_pkg python3 "$PACKAGE_MANAGER"
  fi
}

function welcome_message {
cat <<EOF
Welcome to Diem!

This script will download and install the necessary dependencies needed to
build, test and inspect Diem Core.

Based on your selection, these tools will be included:
EOF

  if [[ "$INSTALL_BUILD_TOOLS" == "true" ]]; then
cat <<EOF
Build tools (since -t or no option was provided):
  * Rust (and the necessary components, e.g. rust-fmt, clippy)
  * CMake
  * Clang
  * grcov
  * lcov
  * pkg-config
  * libssl-dev
  * sccache
  * if linux, gcc-powerpc-linux-gnu
  * NodeJS / NPM
EOF
  fi

  if [[ "$OPERATIONS" == "true" ]]; then
cat <<EOF
Operation tools (since -o was provided):
  * yamllint
  * python3
  * docker
  * vault
  * terraform
  * kubectl
  * helm
  * aws cli
  * allure
EOF
  fi

  if [[ "$INSTALL_PROVER" == "true" ]]; then
cat <<EOF
Move prover tools (since -y was provided):
  * z3
  * cvc4
  * cvc5
  * dotnet
  * boogie
EOF
  fi

  if [[ "$INSTALL_CODEGEN" == "true" ]]; then
cat <<EOF
Codegen tools (since -s was provided):
  * Clang
  * Python3 (numpy, pyre-check)
  * Golang
  * Java
  * Deno
EOF
  fi

  if [[ "$INSTALL_API_BUILD_TOOLS" == "true" ]]; then
cat <<EOF
API build and testing tools (since -a was provided):
  * Python3 (schemathesis)
EOF
  fi

  if [[ "$INSTALL_PROFILE" == "true" ]]; then
cat <<EOF
Moreover, ~/.profile will be updated (since -p was provided).
EOF
  fi

cat <<EOF
If you'd prefer to install these dependencies yourself, please exit this script
now with Ctrl-C.
EOF
}

BATCH_MODE=false;
VERBOSE=false;
INSTALL_BUILD_TOOLS=false;
OPERATIONS=false;
INSTALL_PROFILE=false;
INSTALL_PROVER=false;
INSTALL_CODEGEN=false;
INSTALL_API_BUILD_TOOLS=false;
INSTALL_INDIVIDUAL=false;
INSTALL_PACKAGES=();
INSTALL_DIR="${HOME}/bin/"
OPT_DIR="false"

#parse args
while getopts "btopvysah:i:n" arg; do
  case "$arg" in
    b)
      BATCH_MODE="true"
      ;;
    t)
      INSTALL_BUILD_TOOLS="true"
      ;;
    o)
      OPERATIONS="true"
      ;;
    p)
      INSTALL_PROFILE="true"
      ;;
    v)
      VERBOSE=true
      ;;
    y)
      INSTALL_PROVER="true"
      ;;
    s)
      INSTALL_CODEGEN="true"
      ;;
    a)
      INSTALL_API_BUILD_TOOLS="true"
      ;;
    i)
      INSTALL_INDIVIDUAL="true"
      echo "$OPTARG"
      INSTALL_PACKAGES+=("$OPTARG")
      ;;
    n)
      OPT_DIR="true"
      ;;
    *)
      usage;
      exit 0;
      ;;
  esac
done

if [[ "$VERBOSE" == "true" ]]; then
	set -x
fi

if [[ "$INSTALL_BUILD_TOOLS" == "false" ]] && \
   [[ "$OPERATIONS" == "false" ]] && \
   [[ "$INSTALL_PROFILE" == "false" ]] && \
   [[ "$INSTALL_PROVER" == "false" ]] && \
   [[ "$INSTALL_CODEGEN" == "false" ]] && \
   [[ "$INSTALL_API_BUILD_TOOLS" == "false" ]] && \
   [[ "$INSTALL_INDIVIDUAL" == "false" ]]; then
   INSTALL_BUILD_TOOLS="true"
fi

if [ ! -f rust-toolchain ]; then
	echo "Unknown location. Please run this from the diem repository. Abort."
	exit 1
fi

if [[ "${OPT_DIR}" == "true" ]]; then
  INSTALL_DIR="/opt/bin/"
fi
mkdir -p "$INSTALL_DIR" || true

PRE_COMMAND=()
if [ "$(whoami)" != 'root' ]; then
  PRE_COMMAND=(sudo)
fi

PACKAGE_MANAGER=
if [[ "$(uname)" == "Linux" ]]; then
	if command -v yum &> /dev/null; then
		PACKAGE_MANAGER="yum"
	elif command -v apt-get &> /dev/null; then
		PACKAGE_MANAGER="apt-get"
	elif command -v pacman &> /dev/null; then
		PACKAGE_MANAGER="pacman"
  elif command -v apk &>/dev/null; then
		PACKAGE_MANAGER="apk"
  elif command -v dnf &>/dev/null; then
    echo "WARNING: dnf package manager support is experimental"
    PACKAGE_MANAGER="dnf"
	else
		echo "Unable to find supported package manager (yum, apt-get, dnf, or pacman). Abort"
		exit 1
	fi
elif [[ "$(uname)" == "Darwin" ]]; then
	if command -v brew &>/dev/null; then
		PACKAGE_MANAGER="brew"
	else
		echo "Missing package manager Homebrew (https://brew.sh/). Abort"
		exit 1
	fi
else
	echo "Unknown OS. Abort."
	exit 1
fi

if [[ "$BATCH_MODE" == "false" ]]; then
    welcome_message
    printf "Proceed with installing necessary dependencies? (y/N) > "
    read -e -r input
    if [[ "$input" != "y"* ]]; then
	    echo "Exiting..."
	    exit 0
    fi
fi

if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
	if [[ "$BATCH_MODE" == "false" ]]; then
    echo "Updating apt-get......"
  fi
	"${PRE_COMMAND[@]}" apt-get update
  if [[ "$BATCH_MODE" == "false" ]]; then
   echo "Installing ca-certificates......"
  fi
	install_pkg ca-certificates "$PACKAGE_MANAGER"
fi

if [[ "$INSTALL_PROFILE" == "true" ]]; then
  update_path_and_profile
fi

install_pkg curl "$PACKAGE_MANAGER"


if [[ "$INSTALL_BUILD_TOOLS" == "true" ]]; then
  install_build_essentials "$PACKAGE_MANAGER"
  install_pkg cmake "$PACKAGE_MANAGER"
  install_pkg clang "$PACKAGE_MANAGER"
  install_pkg llvm "$PACKAGE_MANAGER"

  install_gcc_powerpc_linux_gnu "$PACKAGE_MANAGER"
  install_openssl_dev "$PACKAGE_MANAGER"
  install_pkg_config "$PACKAGE_MANAGER"

  install_rustup "$BATCH_MODE"
  install_toolchain "$(cat ./rust-toolchain)"
  # Add all the components that we need
  rustup component add rustfmt
  rustup component add clippy

  install_cargo_guppy
  install_sccache
  install_grcov
  install_pkg git "$PACKAGE_MANAGER"
  install_lcov "$PACKAGE_MANAGER"
  install_nodejs "$PACKAGE_MANAGER"
fi

if [[ "$OPERATIONS" == "true" ]]; then
  install_pkg yamllint "$PACKAGE_MANAGER"
  install_pkg python3 "$PACKAGE_MANAGER"
  install_pkg unzip "$PACKAGE_MANAGER"
  install_pkg jq "$PACKAGE_MANAGER"
  install_pkg git "$PACKAGE_MANAGER"
  install_tidy "$PACKAGE_MANAGER"
  install_xsltproc
  #for timeout
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    install_pkg coreutils "$PACKAGE_MANAGER"
  fi
  install_shellcheck
  install_hadolint
  install_vault
  install_helm
  install_terraform
  install_kubectl
  install_awscli "$PACKAGE_MANAGER"
  install_allure
fi

if [[ "$INSTALL_INDIVIDUAL" == "true" ]]; then
  for (( i=0; i < ${#INSTALL_PACKAGES[@]}; i++ ));
  do
    PACKAGE=${INSTALL_PACKAGES[$i]}
    if ! command -v "install_${PACKAGE}" &> /dev/null; then
      install_pkg "$PACKAGE" "$PACKAGE_MANAGER"
    else
      "install_${PACKAGE}"
    fi
  done
fi

if [[ "$INSTALL_PROVER" == "true" ]]; then
  export DOTNET_INSTALL_DIR="${HOME}/.dotnet/"
  if [[ "$OPT_DIR" == "true" ]]; then
    export DOTNET_INSTALL_DIR="/opt/dotnet/"
    mkdir -p "$DOTNET_INSTALL_DIR" || true
  fi
  install_z3
  install_cvc4
  install_cvc5
  install_dotnet
  install_boogie
fi

if [[ "$INSTALL_CODEGEN" == "true" ]]; then
  install_pkg clang "$PACKAGE_MANAGER"
  install_pkg llvm "$PACKAGE_MANAGER"
  install_python3
  install_deno
  install_java
  install_golang
  if [[ "$PACKAGE_MANAGER" != "apk" ]]; then
    # depends on wheels which needs glibc which doesn't work on alpine's python.
    # Only invested a hour or so in this, a work around may exist.
    "${PRE_COMMAND[@]}" python3 -m pip install pyre-check=="${PYRE_CHECK_VERSION}"
  fi
  "${PRE_COMMAND[@]}" python3 -m pip install numpy=="${NUMPY_VERSION}"
fi

if [[ "$INSTALL_API_BUILD_TOOLS" == "true" ]]; then
  # python and tools
  install_python3
  "${PRE_COMMAND[@]}" python3 -m pip install schemathesis
fi

if [[ "${BATCH_MODE}" == "false" ]]; then
cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
	cargo build
EOF
fi

exit 0
