#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -x;

# run this script from the project root using `./scripts/build_docs.sh`

function usage() {
  echo "Usage: $0 [-b] [-r]"
  echo ""
  echo "Build Libra documentation."
  echo ""
  echo "  -b   Build static version of documentation (otherwise start server)"
  echo ""
  echo "  -r   Build Libra Rust crate documentation"
  echo ""
}

function install_rustup {
  echo "Installing Rust......"
  if rustup --version &>/dev/null; then
    echo "Rust is already installed"
  else
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
    PATH="${HOME}/.cargo/bin:${PATH}"
  fi
}

BUILD_STATIC=false
BUILD_RUSTDOCS=false

if [[ "$(basename $PWD)" != "developers.libra.org" ]]; then
  echo "Didn't pass directory check."
  echo ""
  echo "The script must be run from the developers.libra.org directory via ./scripts/build_docs.sh"
  echo ""
  echo "You are running it from: "
  echo $(echo $(basename $PWD))
  echo ""
  exit 1
fi

while getopts 'hbr' flag; do
  case "${flag}" in
    h)
      usage;
      exit 0;
      ;;
    b)
      BUILD_STATIC=true
      ;;
    r)
      BUILD_RUSTDOCS=true
      ;;
    *)
      usage;
      exit 0;
      ;;
  esac
done

# Install Rust (Netlify will need this for website previews, for example)
install_rustup

# create needed output directories, ignore error if they exist already
mkdir -p docs/crates
mkdir -p docs/community

# manually copy crate README files from fixed directory
###
echo "-----------------------------------"
echo "Manually Copying READMEs to docs/crates"
echo "-----------------------------------"
sed -i.old '/^# /d' ../language/bytecode-verifier/README.md; cp ../language/bytecode-verifier/README.md docs/crates/bytecode-verifier.md
sed -i.old '/^# /d' ../consensus/README.md; cp ../consensus/README.md docs/crates/consensus.md
sed -i.old '/^# /d' ../crypto/crypto/README.md; cp ../crypto/crypto/README.md docs/crates/crypto.md
sed -i.old '/^# /d' ../execution/README.md; cp ../execution/README.md docs/crates/execution.md
sed -i.old '/^# /d' ../language/README.md; cp ../language/README.md docs/crates/move-language.md
sed -i.old '/^# /d' ../language/compiler/README.md; cp ../language/compiler/README.md docs/crates/ir-to-bytecode.md
sed -i.old '/^# /d' ../mempool/README.md; cp ../mempool/README.md docs/crates/mempool.md
sed -i.old '/^# /d' ../network/README.md; cp ../network/README.md docs/crates/network.md
sed -i.old '/^# /d' ../storage/README.md; cp ../storage/README.md docs/crates/storage.md
sed -i.old '/^# /d' ../language/vm/README.md; cp ../language/vm/README.md docs/crates/vm.md

echo "-----------------------------------"
echo "Manually Copy Coding Guidelines"
echo "-----------------------------------"
sed -i.old '/^# Libra Core Coding Guidelines/d' ../documentation/coding_guidelines.md
cp ../documentation/coding_guidelines.md docs/community/coding-guidelines.md

echo "-----------------------------------"
echo "Manually Copy Contributing Guidelines"
echo "-----------------------------------"
sed -i.old '/^# Libra Core Contributing Guidelines/d' ../CONTRIBUTING.md
cp ../CONTRIBUTING.md docs/community/contributing.md

if [[ $BUILD_RUSTDOCS == true ]]; then
  echo "-----------------------------------"
  echo "Generating API reference via Rustdoc"
  echo "-----------------------------------"

  # Back to the Libra repo root dir
  cd ..

  # Build the rust crate docs
  # Use `RUSTC_BOOTSTRAP` in order to use the `--enable-index-page` flag of rustdoc
  # This is needed in order to generate a landing page `index.html` for workspaces
  export PATH="$PATH:$HOME/.cargo/bin"
  RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --enable-index-page" cargo doc --no-deps --workspace --lib || exit 1
  # Use the '.' to make sure we only copy the content from the doc dir, not the doc dir itself too.
  # Avoids having developers.libra.org/docs/rustdocs/doc. We want developers.libra.org/docs/rustdocs/
  RUSTDOC_DIR='../target/doc/.'
  DOCUSAURUS_RUSTDOC_DIR='website/static/docs/rustdocs/'
  cd developers.libra.org || exit

  mkdir -p $DOCUSAURUS_RUSTDOC_DIR
  cp -r $RUSTDOC_DIR $DOCUSAURUS_RUSTDOC_DIR
fi

echo "-----------------------------------"
echo "Building Docusaurus 🦖"
echo "-----------------------------------"
cd website || exit
npm install

if [[ $BUILD_STATIC == true ]]; then
  echo "-----------------------------------"
  echo "Building static site"
  echo "-----------------------------------"
  npm run build
else
  echo "-----------------------------------"
  echo "Starting local server"
  echo "-----------------------------------"
  npm run start
fi
