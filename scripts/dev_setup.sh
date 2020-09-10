#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
# This script sets up the environment for the Libra build by installing necessary dependencies.
#
# Usage ./dev_setup.sh <options>
#   v - verbose, print all statements

# Assumptions for nix systems:
# 1 The running user is the user who will execute the builds.
# 2 .profile will be used to configure the shell
# 3 ${HOME}/bin/ is expected to be on the path - hashicorp tools/hadolint/etc.  will be installed there on linux systems.

SHELLCHECK_VERSION=0.7.1
HADOLINT_VERSION=1.17.4
SCCACHE_VERSION=0.2.13
KUBECTL_VERSION=1.18.6
TERRAFORM_VERSION=0.12.26
HELM_VERSION=3.2.4
VAULT_VERSION=1.5.0

SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

function usage {
  echo "Usage:"
  echo "Installs or updates necessary dev tools for libra/libra."
  echo "-b batch mode, no user interactions and miminal output"
  echo "-p update ${HOME}/.profile"
  echo "-o intall operations tooling as well: helm, terraform, hadolint, yamllint, vault, docker, kubectl, python3"
  echo "-v verbose mode"
  echo "should be called from the root folder of the libra project"
}

function update_path_and_profile {
  touch "${HOME}"/.profile
  mkdir -p "${HOME}"/bin
  export PATH="${HOME}"/bin:"${HOME}"/.cargo/bin:"${PATH}"
  FOUND=$(grep -c "export PATH=\"${HOME}/bin:${HOME}/.cargo/bin:\$PATH\"" < "${HOME}/.profile")
  if [ "$FOUND" == "0" ]; then
    echo "export PATH=\"${HOME}/bin:${HOME}/.cargo/bin:\$PATH\"" >> "${HOME}"/.profile
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
  if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
    install_pkg gcc "$PACKAGE_MANAGER"
    install_pkg gcc-c++ "$PACKAGE_MANAGER"
    install_pkg make "$PACKAGE_MANAGER"
  fi
  #if [[ "$PACKAGE_MANAGER" == "brew" ]]; then
  #  install_pkg pkgconfig "$PACKAGE_MANAGER"
  #fi
}

function install_rustup {
  BATCH_MODE=$1
  # Install Rust
  [[ "${BATCH_MODE}" == "false" ]] && echo "Installing Rust......"
  if rustup --version &>/dev/null; then
	   [[ "${BATCH_MODE}" == "false" ]] && echo "Rust is already installed"
  else
	  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
    PATH="${HOME}/.cargo/bin:${PATH}"
  fi
}

function install_hadolint {
  if ! command -v hadolint &> /dev/null; then
    export HADOLINT=${HOME}/bin/hadolint
    curl -sL -o "$HADOLINT" "https://github.com/hadolint/hadolint/releases/download/v${HADOLINT_VERSION}/hadolint-$(uname -s)-$(uname -m)" && chmod 700 "$HADOLINT"
  fi
  hadolint -v
}

function install_vault {
  VERSION=$(vault --version)
  if [[ "$VERSION" != "Vault v${VAULT_VERSION}" ]]; then
    MACHINE=$(uname -m);
    if [[ $MACHINE == "x86_64" ]]; then
      MACHINE="amd64"
    fi
    TMPFILE=$(mktemp)
    curl -sL -o "$TMPFILE" "https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_$(uname -s | tr '[:upper:]' '[:lower:]')_${MACHINE}.zip"
    unzip -qq -d "${HOME}"/bin/ "$TMPFILE"
    rm "$TMPFILE"
    chmod +x "${HOME}"/bin/vault
  fi
  vault --version
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
      cp "${TMPFILE}/$(uname -s | tr '[:upper:]' '[:lower:]')-${MACHINE}/helm" "${HOME}/bin/helm"
      rm -rf "$TMPFILE"
      chmod +x "${HOME}"/bin/helm
    fi
  fi
}

function install_terraform {
  VERSION=$(terraform --version | head -1)
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
      unzip -qq -d "${HOME}"/bin/ "$TMPFILE"
      rm "$TMPFILE"
      chmod +x "${HOME}"/bin/terraform
      terraform --version
    fi
  fi
}

function install_kubectl {
  VERSION=$(kubectl version client --short=true | head -1)
  if [[ "$VERSION" != "Client Version: v${KUBECTL_VERSION}" ]]; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg kubectl brew
    else
      MACHINE=$(uname -m);
      if [[ $MACHINE == "x86_64" ]]; then
        MACHINE="amd64"
      fi
      curl -sL -o "${HOME}"/bin/kubectl "https://storage.googleapis.com/kubernetes-release/release/v${KUBECTL_VERSION}/bin/$(uname -s | tr '[:upper:]' '[:lower:]')/${MACHINE}/kubectl"
      chmod +x "${HOME}"/bin/kubectl
    fi
  fi
  kubectl version client --short=true | head -1
}

function install_awscli {
  if ! command -v aws &> /dev/null; then
    if [[ $(uname -s) == "Darwin" ]]; then
      install_pkg awscli brew
    else
      MACHINE=$(uname -m);
      TMPFILE=$(mktemp)
      rm "$TMPFILE"
      mkdir -p "$TMPFILE"/work/
      curl -sL -o "$TMPFILE"/aws.zip  "https://awscli.amazonaws.com/awscli-exe-$(uname -s | tr '[:upper:]' '[:lower:]')-${MACHINE}.zip"
      unzip -qq -d "$TMPFILE"/work/ "$TMPFILE"/aws.zip
      mkdir -p "${HOME}"/.local/
      "$TMPFILE"/work/aws/install -i "${HOME}"/.local/aws-cli -b "${HOME}"/bin
      rm -rf "$TMPFILE"
    fi
  fi
  aws --version
}

function install_pkg {
  package=$1
  PACKAGE_MANAGER=$2
  PRE_COMMAND=()
  if [ "$(whoami)" != 'root' ]; then
    PRE_COMMAND=(sudo)
  fi
  if which "$package" &>/dev/null; then
    echo "$package is already installed"
  else
    echo "Installing ${package}."
    if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
      "${PRE_COMMAND[@]}" yum install "${package}" -y
    elif [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      "${PRE_COMMAND[@]}" apt-get install "${package}" --no-install-recommends -y
    elif [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
      "${PRE_COMMAND[@]}" pacman -Syu "$package" --noconfirm
    elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
      apk --update add --no-cache "${package}"
    elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
      brew install "$package"
    fi
  fi
}

function install_pkg_config {
  PACKAGE_MANAGER=$1
  #Differently named packages for pkg-config
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
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
      cp "${TMPFILE}/shellcheck-v${SHELLCHECK_VERSION}/shellcheck" "${HOME}/bin/shellcheck"
      rm -rf "$TMPFILE"
      chmod +x "${HOME}"/bin/shellcheck
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
  if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
    install_pkg openssl-devel "$PACKAGE_MANAGER"
  fi
  if [[ "$PACKAGE_MANAGER" == "pacman" ]] || [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    install_pkg openssl "$PACKAGE_MANAGER"
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
  FOUND=$(rustup show | grep -c "$version" )
  if [[ "$FOUND" == "0" ]]; then
    echo "Installing ${version} of rust toolchain"
    rustup install "$version"
  else
    echo "${version} rust toolchain already installed"
  fi
}

function install_sccache {
  VERSION="$(sccache --version)"
  if [[ "$VERSION" != "sccache ""${SCCACHE_VERSION}" ]]; then
    cargo install sccache --version="${SCCACHE_VERSION}"
  fi
}

function install_grcov {
  if ! command -v grcov &> /dev/null; then
    cargo install grcov
  fi
}

function welcome_message {
cat <<EOF
Welcome to Libra!

This script will download and install the necessary dependencies needed to
build, test and inspect Libra Core. This includes:
  * Rust (and the necessary components, e.g. rust-fmt, clippy)
  * CMake
  * Clang
  * grcov
  * lcov
  * pkg-config
  * libssl-dev
  * sccache
  * if linux, gcc-powerpc-linux-gnu
If operations tools are selected, then
  * yamllint
  * python3
  * docker
  * vault
  * terraform
  * kubectl
  * helm
  * aws cli

If you'd prefer to install these dependencies yourself, please exit this script
now with Ctrl-C.
EOF
}

BATCH_MODE=false;
VERBOSE=false;
OPERATIONS=false;
INSTALL_PROFILE=false;

#parse args
while getopts "bopvh" arg; do
  case "$arg" in
    b)
      BATCH_MODE="true"
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
    *)
      usage;
      exit 0;
      ;;
  esac
done

if [[ "$VERBOSE" == "true" ]]; then
	set -x
fi

if [ ! -f rust-toolchain ]; then
	echo "Unknown location. Please run this from the libra repository. Abort."
	exit 1
fi

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
	else
		echo "Unable to find supported package manager (yum, apt-get, or pacman). Abort"
		exit 1
	fi
elif [[ "$(uname)" == "Darwin" ]]; then
	if which brew &>/dev/null; then
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
	[[ "$BATCH_MODE" == "false" ]] && echo "Updating apt-get......"
	"${PRE_COMMAND[@]}" apt-get update
fi

[[ "$INSTALL_PROFILE" == "true" ]] && update_path_and_profile

install_pkg curl "$PACKAGE_MANAGER"
install_shellcheck

install_build_essentials "$PACKAGE_MANAGER"
install_pkg cmake "$PACKAGE_MANAGER"
install_pkg clang "$PACKAGE_MANAGER"
install_pkg llvm "$PACKAGE_MANAGER"


install_gcc_powerpc_linux_gnu "$PACKAGE_MANAGER"
install_openssl_dev "$PACKAGE_MANAGER"
install_pkg_config "$PACKAGE_MANAGER"

install_rustup "$BATCH_MODE"
install_toolchain "$(cat ./cargo-toolchain)"
install_toolchain "$(cat ./rust-toolchain)"
# Add all the components that we need
rustup component add rustfmt
rustup component add clippy

install_sccache
install_grcov
install_pkg lcov "$PACKAGE_MANAGER"

if [[ "$OPERATIONS" == "true" ]]; then
  install_pkg yamllint "$PACKAGE_MANAGER"
  install_pkg python3 "$PACKAGE_MANAGER"
  install_pkg unzip "$PACKAGE_MANAGER"
  install_shellcheck
  install_hadolint
  install_vault
  install_helm
  install_terraform
  install_kubectl
  install_awscli
fi

[[ "${BATCH_MODE}" == "false" ]] && cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
	cargo build
EOF

exit 0
