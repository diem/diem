# On-chain Discovery Protocol

## Overview

The [LibraNet](spec.md) On-chain Discovery Protocol is an authenticated discovery protocol for nodes to learn validator and VFN network addresses and network identity public keys. On-chain discovery leverages the Move language and Libra blockchain to serve as a central authenticated data-store for distributing advertised validator and VFN discovery information in the form of [`RawNetworkAddress`](network-address.md)es.

## Design Principles

* All communication between peers _MUST_ be authenticated and encrypted.
* All authentication is rooted in the Libra blockchain. In the same way that the chain is the source-of-truth for the `ValidatorSet`, the chain is also the source of truth for validator and VFN network addresses and network identity public keys.
* Uniform discovery interface for both validators and VFNs.
* Unify key rotation and address rotation procedures. The protocol for adding/removing a validator or rotating the consensus key is effectively the same as rotating the network key or network addresses.
* Support network protocol upgradability.

## Use cases

There are four separate discovery problems in Libra:

1. Validators discovering other validators.
2. All node types discovering public-facing VFNs.
3. Discovering the nodes inside a validator's private, internal cluster.
4. Discovering PFNs

On-chain discovery serves use cases (1) and (2) but not (3) or (4).

## On-chain Config

(TODO(philiphayes): describe encryption of validator addresses)

Validator and VFN discovery information are stored in the `ValidatorSet` in the [`OnChainConfig`](../consensus/spec.md#onchainconfig).

```rust
struct ValidatorSet {
    scheme: ConsensusScheme,
    payload: Vec<ValidatorInfo>,
}

struct ValidatorInfo {
    // The validator's account address. AccountAddresses are initially derived from the account
    // auth pubkey; however, the auth key can be rotated, so one should not rely on this
    // initial property.
    account_address: AccountAddress,
    // Voting power of this validator
    consensus_voting_power: u64,
    // Validator config
    config: ValidatorConfig,
}

struct ValidatorConfig {
    consensus_public_key: Ed25519PublicKey,
    validator_network_addresses: Vec<RawNetworkAddress>,
    full_node_network_addresses: Vec<RawNetworkAddress>,
}

#[repr(u8)]
enum ConsensusScheme {
    Ed25519 = 0,
}
```

* [`AccountAddress`](../common/common_data_structures.md#accountaddress)
* [`RawNetworkAddress`](network-address.md)

## LibraNet Messages

```rust
/// OnchainDiscovery LibraNet message types. These are sent over-the-wire.
enum OnchainDiscoveryMsg {
    QueryDiscoverySetRequest(QueryDiscoverySetRequest),
    QueryDiscoverySetResponse(QueryDiscoverySetResponse),
}

/// A request for another peer's latest validator set and a validator change proof
/// to get the client up-to-date if they're behind.
struct QueryDiscoverySetRequest {
    known_version: u64,
}

/// A response to a [`QueryDiscoverySetRequest`]. The server will include an
/// epoch change proof if the client is behind.
///
/// The validator set only changes when there is a new epoch. To minimize
/// wire overhead, the server will include the validator set account's
/// [`AccountStateWithProof`] _if and only if_ the server also presents a
/// non-empty epoch change proof.
struct QueryDiscoverySetResponse {
    latest_li: LedgerInfoWithSignatures,
    epoch_change_proof: EpochChangeProof,
    accumulator_proof: AccumulatorConsistencyProof,
    account_state: Option<AccountStateWithProof>,
}
```

* [`LedgerInfoWithSignatures`](../common/common_data_structures.md#ledgerinfowithsignatures)
* [`EpochChangeProof`](../consensus/spec.md#epochchangeproof)
* `AccumulatorConsistencyProof` (TODO(philiphayes): link)
* `AccountStateWithProof` (TODO(philiphayes): link)

(TODO(philiphayes): update the message types after we migrate to the new storage API)
(TODO(philiphayes): how to deal with partial epoch change proof?)

Validators and VFNs service `QueryDiscoverySetRequest`s. In the future, this bespoke service will probably move to a more general storage query API. Note that validators will only service other validators' requests.

## Bootstrapping

Nodes bootstrap onto the network using the latest known validator set from their latest known chain state in storage (which may be the genesis state) and seed peers from their local configuration if they are too far behind. So long as at least one peer is available that will accept the bootstrapping node's connection and will service their `QueryDiscoverySetRequest`, then the bootstrapping node will successfully ratchet up to the latest epoch and learn the `ValidatorSet` for that epoch.

## Steady State

Once nodes are up-to-date, they can receive updates to the on-chain validator set from their own state-sync module. Otherwise, they must actively query their peers for the latest validator set to ensure they have the latest connectivity information for all their peers.

## Network Key and Address Rotation

On-chain discovery supports several different key and address rotation patterns, though the general theme looks like:

1. Submit a transaction that sets the validator's `ValidatorInfo` to the desired state.
2. Wait for the transaction to commit and trigger a reconfiguration. If the transaction fails for whatever reason, then the rotation has failed.
3. When the validator observes the reconfiguration and sees the change to its advertised addresses, it can locally update its set of listening addresses and/or pubkeys it will use to dial out.

If a node operator is manually rotating a validator's key or address, then they should manually restart the validator with the new keypair or address configuration in step (3.) after observing the epoch change.

### Automated Key Rotation

Ideally, routine key rotations are automated and don't require operator intervention. An optional procedure for automated key rotation is outlined below:

Imagine a validator starts with a single advertised network address containing its network identity public key `<pubkey1>`:

```
addrs = ["/ip4/1.2.3.4/tcp/6180/ln-noise-ik/<pubkey1>/ln-handshake/0"]
```

The validator inititates a key rotation to a new network identity public key `<pubkey2>` by sending a transaction to set its addresses to a new list:

```
tx: set_validator_network_addresses(["/ip4/1.2.3.4/tcp/6180/ln-noise-ik/<pubkey2>/ln-handshake/0"])
```

(TODO(philiphayes): link to actual tx?)

When the transaction commits, the validator observes a reconfiguration with its new advertised network address. It will then begin responding to noise handshakes with the new keypair. Likewise, the node will use the new keypair when dialing out to other peers.

There are, however, some edge cases that require careful consideration. For example, suppose that the validator submits its rotation tx but then crashes for a bit or gets partitioned from the network before observing the reconfiguration. Other nodes then observe the reconfiguration and stop accepting new connections for its old public key. This situation would be problematic, as the validator can no longer connect to any of the other validators (since its old pubkey is no longer trusted). Fortunately, the validator should be able to learn about the most recent reconfiguration by epoch sync'ing from any public-facing VFN endpoints, which do not discriminate connections by public key.

Alternatively, a safer approach to preserve validator connectivity (at the expense of more complexity) might be to rotate in 2 steps. Given the same setup as before, the validator can instead advertise both `<pubkey1>` and `<pubkey2>` simultaneously. Only once it has observed this reconfiguration does it rotate to advertising only `<pubkey2>`.

## Caveats

1. The current on-chain model for on-chain discovery scales poorly for very large validator sets. Due to the way account state proofs work, query proofs require the entire account state blob (containing every validator's discovery information) in order to verify the proof. In other words, nodes can't query a subset of the discovery information and still verify the proof-of-inclusion.

2. Modifications to the discovery information requires a quorum. In the event of a connectivity crisis where the validator set loses quorum (e.g. 1/3+ validators crash and forget their identity pubkeys), validators can't submit transactions to modify the on-chain discovery information to regain connectivity. A sufficient fallback in such an extreme event might be for each validator to manually configure their seed peers config with all other validators' discovery information. Alternatively, the Libra Association may issue a new Genesis Transaction to manually set a new validator set, though this requires a hard fork and significant coordination.

(TODO(philiphayes): link to LUP when spec exists)
