> **Note to readers:** Silvergate Capital Corporation announced in January 2022 that it acquired intellectual property and other technology assets related to running a blockchain-based payment network from Diem, further investing in its platform and enhancing its existing stablecoin infrastructure.
<a href="https://developers.diem.com">
	<img width="200" src="./.assets/diem.png" alt="Diem Logo" />
</a>

---

[![Diem Rust Crate Documentation (main)](https://img.shields.io/badge/docs-main-59f)](https://diem.github.io/diem/)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![grcov](https://img.shields.io/badge/Coverage-grcov-green)](https://ci-artifacts.diem.com/coverage/unit-coverage/latest/index.html)
[![test history](https://img.shields.io/badge/Test-History-green)](https://ci-artifacts.diem.com/testhistory/diem/diem/auto/ci-test.yml/index.html)
[![Automated Issues](https://img.shields.io/github/issues-search?color=orange&label=Automated%20Issues&query=repo%3Adiem%2Fdiem%20is%3Aopen%20author%3Aapp%2Fgithub-actions)](https://github.com/diem/diem/issues/created_by/app/github-actions)
[![Discord chat](https://img.shields.io/discord/903339070925721652.svg?logo=discord&style=flat-square)](https://discord.gg/epNwRT2wcd)


Diem Core implements a decentralized, programmable database which provides a financial infrastructure that can empower billions of people.

## Note to Developers
* Diem Core is a prototype.
* The APIs are constantly evolving and designed to demonstrate types of functionality. Expect substantial changes before the release.
* We’ve launched a testnet that is a live demonstration of an early prototype of the Diem Blockchain software.

## Getting Started

### Learn About Diem
* [Welcome](https://developers.diem.com/docs/welcome-to-diem)
* [Basic Concepts](https://developers.diem.com/docs/basics/basics-txns-states)
* [Life of a Transaction](https://developers.diem.com/docs/transactions/basics-life-of-txn)
* [JSON-RPC SPEC](json-rpc/json-rpc-spec.md)

### Try Diem Core
* [My First Transaction](https://developers.diem.com/docs/tutorials/tutorial-my-first-transaction)
* [Getting Started With Move](https://diem.github.io/move/introduction.html)

### Technical Papers
* [The Diem Blockchain](https://developers.diem.com/docs/technical-papers/the-diem-blockchain-paper)
* [Move: A Language With Programmable Resources](https://developers.diem.com/docs/technical-papers/move-paper)
* [State Machine Replication in the Diem Blockchain](https://developers.diem.com/docs/technical-papers/state-machine-replication-paper)

### Blog
* [Diem: The Path Forward](https://developers.diem.com/blog/2019/06/18/the-path-forward/)

## Building The Source

### **1. Download the source code.**

```bash
$ git clone https://github.com/diem/diem.git
$ cd diem
```

### **2. Install dependencies.**

Run the setup script to install the dependencies required by Diem Core. The setup script performs these actions:
* Installs rustup, an installer for the Rust programming language, in which Diem Core is implemented.
* Installs the required versions of the rust-toolchain.
* Installs CMake to manage the build process.

```bash
$ ./scripts/dev_setup.sh
```

### **3. Build.**

```bash
$ cargo build
```

### **4. Run the test suite.**

```bash
$ cargo test
```

## License

Diem Core is licensed as [Apache 2.0](https://github.com/diem/diem/blob/main/LICENSE).
