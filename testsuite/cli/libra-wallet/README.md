# Libra Wallet

Libra Wallet is a pure-rust implementation of hierarchical key derivation for SecretKey material in Libra.

# Overview

`libra-wallet` is a library providing hierarchical key derivation for SecretKey material in Libra. The following crate is largely inspired by [`rust-wallet`](https://github.com/rust-bitcoin/rust-wallet) with minor modifications to the key derivation function. Note that Libra makes use of the ed25519 Edwards Curve Digital Signature Algorithm (EdDSA) over the Edwards Cruve cruve25519. Therefore, BIP32-like PublicKey derivation is not possible without falling back to a traditional non-deterministic Schnorr signature algorithm. For this reason, we modified the key derivation function to a simpler alternative.

The `internal_macros.rs` is taken from [`rust-bitcoin`](https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/internal_macros.rs)  and `mnemonic.rs` is a slightly modified version of the file with the same name from [`rust-wallet`](https://github.com/rust-bitcoin/rust-wallet/blob/master/src/mnemonic.rs), while `error.rs`, `key_factor.rs` and `wallet_library.rs` are modified to present a minimalist wallet library for the Libra Client. Note that `mnemonic.rs` from `rust-wallet` adheres to the [`BIP39`](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki) spec.

# Implementation Details

`key_factory.rs` implements the key derivation functions. The `KeyFactory` struct holds the Master Secret Material used to derive the Child Key(s). The constructor of a particular `KeyFactory` accepts a `[u8; 64]` `Seed` and computes both the `Master` Secret Material as well as the `ChainCode` from the HMAC-512 of the `Seed`. Finally, the `KeyFactory` allows to derive a child PrivateKey at a particular `ChildNumber` from the Master and ChainCode, as well as the `ChildNumber`'s u64 member.

`wallet_library.rs` is a thin wrapper around `KeyFactory` which enables to keep track of Libra `AccountAddresses` and the information required to restore the current wallet from a `Mnemonic` backup. The `WalletLibrary` struct includes constructors that allow to generate a new `WalletLibrary` from OS randomness or generate a `WalletLibrary` from an instance of `Mnemonic`. `WalletLibrary` also allows to generate new addresses in-order or out-of-order via the `fn new_address` and `fn new_address_at_child_number`. Finally, `WalletLibrary` is capable of signing a Libra `RawTransaction` with the PrivateKey associated to the `AccountAddress` submitted. Note that in the future, Libra will support rotating authentication keys and therefore, `WalletLibrary` will need to understand more general inputs when mapping `AuthenticationKeys` to `PrivateKeys`
