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
# 3 ${HOME}/bin/ is expected to be on the path - hashicorp tools/hadolint/etc.  will be installed there on linux systems.

SHELLCHECK_VERSION=0.7.1
HADOLINT_VERSION=1.17.4
SCCACHE_VERSION=0.2.14-alpha.0
#If installing sccache from a git repp set url@revision.
SCCACHE_GIT='https://github.com/rexhoffman/sccache.git@549babdd3866aa60dae01668c42ee00bf1e8c763'
KUBECTL_VERSION=1.18.6
TERRAFORM_VERSION=0.12.26
HELM_VERSION=3.2.4
VAULT_VERSION=1.5.0
Z3_VERSION=4.8.9
CVC4_VERSION=aac53f51
DOTNET_VERSION=3.1
BOOGIE_VERSION=2.7.35

SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

function usage {
  echo "Usage:"
  echo "Installs or updates necessary dev tools for diem/diem."
  echo "-b batch mode, no user interactions and miminal output"
  echo "-p update ${HOME}/.profile"
  echo "-t install build tools"
  echo "-o install operations tooling as well: helm, terraform, hadolint, yamllint, vault, docker, kubectl, python3"
  echo "-y installs or updates Move prover tools: z3, cvc4, dotnet, boogie"
  echo "-v verbose mode"
  echo "If no toolchain component is selected with -t, -o, -y, or -p, the behavior is as if -t had been provided."
  echo "This command must be called from the root folder of the Diem project."
}

function add_to_profile {
  eval "$1"
  FOUND=$(grep -c "$1" < "${HOME}/.profile")
  if [ "$FOUND" == "0" ]; then
    echo "$1" >> "${HOME}"/.profile
  fi
}

function update_path_and_profile {
  touch "${HOME}"/.profile
  mkdir -p "${HOME}"/bin
  add_to_profile "export PATH=\"${HOME}/bin:${HOME}/.cargo/bin:\$PATH\""
  if [[ "$INSTALL_PROVER" == "true" ]]; then
     add_to_profile "export DOTNET_ROOT=\$HOME/.dotnet"
     add_to_profile "export PATH=\"${HOME}/.dotnet/tools:\$PATH\""
     add_to_profile "export Z3_EXE=$HOME/bin/z3"
     add_to_profile "export CVC4_EXE=$HOME/bin/cvc4"
     add_to_profile "export BOOGIE_EXE=$HOME/.dotnet/tools/boogie"
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
  if [[ "$PACKAGE_MANAGER" == "yum" ]] || [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
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
    if [[ -n "${SCCACHE_GIT}" ]]; then
      git_repo=$( echo "$SCCACHE_GIT" | cut -d "@" -f 1 );
      git_hash=$( echo "$SCCACHE_GIT" | cut -d "@" -f 2 );
      cargo install sccache --git "$git_repo" --rev "$git_hash" --features s3;
    else
      cargo install sccache --version="${SCCACHE_VERSION}" --features s3;
    fi
  fi
}

function install_grcov {
  if ! command -v grcov &> /dev/null; then
    cargo install grcov
  fi
}

function install_dotnet {
  echo "Installing .Net"
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
       | TERM=linux /bin/bash -s -- --channel $DOTNET_VERSION --version latest
}

function install_boogie {
  echo "Installing boogie"
  export DOTNET_ROOT=$HOME/.dotnet
  if [[ "$("$HOME"/.dotnet/dotnet tool list -g)" =~ .*boogie.*${BOOGIE_VERSION}.* ]]; then
    echo "Boogie $BOOGIE_VERSION already installed"
  else
    "$HOME/.dotnet/dotnet" tool update --global Boogie --version $BOOGIE_VERSION
  fi
}

function install_z3 {
  echo "Installing Z3"
  if which /usr/local/bin/z3 &>/dev/null; then
    echo "z3 already exists at /usr/local/bin/z3"
    echo "but this install will go to $HOME/bin/z3."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if which "$HOME/bin/z3" &>/dev/null && [[ "$("$HOME/bin/z3" --version)" =~ .*${Z3_VERSION}.* ]]; then
     echo "Z3 ${Z3_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-ubuntu-16.04"
  elif [[ "$(uname)" == "Darwin" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-osx-10.14.6"
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
    cp "$Z3_PKG/bin/z3" "$HOME/bin"
    chmod +x "$HOME/bin/z3"
  )
  rm -rf "$TMPFILE"
}

function install_cvc4 {
  echo "Installing CVC4"
  if which /usr/local/bin/cvc4 &>/dev/null; then
    echo "cvc4 already exists at /usr/local/bin/cvc4"
    echo "but this install will go to $HOME/bin/cvc4."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if which "$HOME/bin/cvc4" &>/dev/null && [[ "$("$HOME/bin/cvc4" --version)" =~ .*${CVC4_VERSION}.* ]]; then
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
    cp "$CVC4_PKG/cvc4" "$HOME/bin"
    chmod +x "$HOME/bin/cvc4"
  )
  rm -rf "$TMPFILE"
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
EOF
  fi

  if [[ "$INSTALL_PROVER" == "true" ]]; then
cat <<EOF
Move prover tools (since -y was provided):
  * z3
  * cvc4
  * dotnet
  * boogie
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

#parse args
while getopts "btopvyh" arg; do
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
   [[ "$INSTALL_PROVER" == "false" ]]; then
   INSTALL_BUILD_TOOLS="true"
fi

if [ ! -f rust-toolchain ]; then
	echo "Unknown location. Please run this from the diem repository. Abort."
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
  elif command -v dnf &>/dev/null; then
    echo "WARNING: dnf package manager support is experimental"
    PACKAGE_MANAGER="dnf"
	else
		echo "Unable to find supported package manager (yum, apt-get, dnf, or pacman). Abort"
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


if [[ "$INSTALL_BUILD_TOOLS" == "true" ]]; then
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
fi

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

if [[ "$INSTALL_PROVER" == "true" ]]; then
  install_z3
  install_cvc4
  install_dotnet
  install_boogie
fi

[[ "${BATCH_MODE}" == "false" ]] && cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
	cargo build
EOF

exit 0
