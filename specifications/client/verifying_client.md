# Diem Verifying Client Spec

A client is expected to be connecting to a full node and pulling its data from that full node. This means that a client has to chose between blindly trusting the full node answers, or verifying them. This specification specifies how a client can verify the full node answers, provided it relies on the so-called verifying API instead of the simpler “non-verifying” API.

## Data serialization 

Serialization is done using BCS (https://docs.rs/bcs/) it should be implemented in order to communicate with the verifying API endpoint, but also in order to verify the proofs, since these assume BCS encoded data sometimes.

## Verifying signatures

We refer to the [Diem Crypto spec](https://github.com/diem/diem/blob/master/specifications/crypto/README.md#signing-and-verifying) for more details regarding how the verification is to be handled. Notice this requires implementing BCS encoding, SHA3-256 as well as Ed25519 verification.

## Alternative to verifying proofs

Instead of verifying proofs as described in this specification, a client could also:

* query randomly multiple nodes and only accept an answer if it is the one given by the majority of the nodes, thus using a consensus-based approach. Notice that this approach is not necessarily the best as anyone can run a full-node, and clients are only connecting to full-nodes, not directly to validators. In the naive case where only Validator Full Nodes are queried, provided that more than 50% of them are exposing a public interface, this is equivalent to trusting the validators, but this is not necessarily going to be the case in the Diem ecosystem.
* run its own full node in order to fully verify everything (this is the recommended way for VASPs, DDs and LPs)

## Trust assumptions / issues

If a client connects to a single full node and pulls all of the data from the full nodes what guarantees the client that the full nodes hasn’t been running a virtual network from Genesis onwards? → The set of initial validators is fixed and the epoch change (and thus change of validator sets) have to be signed by the previous set.

# Server interface

The clients are expected to connect to a public full node. Public full nodes are synchronising their states from the validator full nodes and are communicating with each other to transmit the latest states. They also verify the proofs of everything they receive.
Public full nodes can choose to expose their JSON-RPC endpoints to allow clients to query these using .

## JSON-RPC Endpoint

We assume that clients are querying full nodes using the Full Nodes’ JSON-RPC interface.
Currently verification is only supported by the following (experimental) APIs:

* `get_state_proof`, which allows to get the latest ledger state along with possibly `ValidatorChangeProof` if required.
* `get_account_state_with_proof`, which allows to get the state of an account along with the necessary proofs to verify it.
* `get_transactions_with_proofs`, which 
* `get_events_with_proofs`

Notice that these endpoints should be enough to run a light client that would have fully verified proofs, without any need to trust any given full node.

You can find the JSON-RPC specification here:  https://github.com/diem/diem/blob/master/json-rpc/json-rpc-spec.md

## Bootstrapping

There are two ways of bootstrapping a client software:

* through a hardcoded Genesis and initial validator set which allow a client to verify all epoch changes since the beginning of the chain and to reach the current state of the blockchain with the assurance it has the right data. Notice that this method is subject to the so-called “[long-range attack](https://eprint.iacr.org/2019/1440.pdf)” in the case where the historic validators keys are compromised, one could live in a fake fork of the network even if everything has been verified from the Genesis.
* through a waypoint that represents a coarse snapshot of the blockchain state at some point and which allow to catch up more easily since it doesn’t require verifying all of the epoch changes from the genesis. A waypoint must come from a fully trusted source through a secure channel.

## Waypoints

Waypoints are also a good thing to verify against even in the case of a cold boot from Genesis, as they can allow to detect a “long range attack”.
Waypoints can also be used as a disaster recovery method, as they allow to basically “re-start” from a given state but they are a double-edged sword since an untrusted waypoint can mislead you into being on a fake fork.
Waypoint are not proven, nor verified. They are trusted by default, just like the Genesis.

It is enough for the Waypoint to include the information about a LedgerInfo on an epoch boundary (the LedgerInfo that must include a non-empty validator set for the next epoch).

The waypoint representation is a short string that can be easily included in a config or copied from a website. Notice it is crucial this source must be fully trustable and trusted. It is made of two fields:

* the version, `version: u64`, of the reconfiguration transaction that is being approved by this waypoint
* the hash of the chosen fields of LedgerInfo (including the next validator set),  `value: HashValue`


The fields of a LedgerInfo that are included in the waypoint value are `epoch`, `root_hash`, `version`, `timestamp`, and `next_validator_set`. 
Note that we do not include all the fields because they might not all be deterministic: different clients might observe different values of e.g., `consensus_block_id` of the latest `LedgerInfo` in an epoch.

The textual representation of a Waypoint is  "version:hex encoding of the root hash". For example, the current testnet genesis waypoint is:

> `"0:3139c30efe6dbde4228efb9df32c137dc3a2490b97ab6a086897be1d0cb336f0"`

and it is accessible at https://testnet.diem.com/waypoint.txt 

Waypoints can be used to verify the latest LedgerInfo even if we started from Genesis by comparing the result of getting up to date using a waypoint with the currently known states, thus providing extra confidence that we are on the “right” chain.

You can read more about Waypoints in [this tech paper](http://documentation/tech-papers/lbft-verification/lbft-verification.pdf).

### Catchup to the latest LedgerInfo using a Waypoint

In order to catchup to the latest state and verify it, one needs to get a waypoint, then query its Full Node server for the state at the time of that Waypoint.
The LedgerInfo returned by the FullNode in such a request should match the hash contained in the Waypoint, which must be recomputed locally. If it matches, then this LedgerInfo is “trusted” along with the validators set it contains. This validator set should allow to verify the signatures of the subsequent LedgerInfo received and therefore to query the latest state to the Full Node while verifying the epoch change proofs and signatures. Refer to the “Epoch change proofs validation" section to have details about how this should be done.
[Image: Screenshot 2020-12-23 at 19.33.50.png]
## Epoch change proofs validation

In order to verify the LedgerInfo state we receive from full nodes, we need to verify that their signatures verify against the current validator set public keys. 
But the validator set can change during the lifecycle of the blockchain. Whenever the validator set changes, this triggers a so-called “epoch change” through a reconfiguration transaction. In order to verify the latest state, one needs to make sure all of these epoch changes are legit. This is achieved using the `ValidatorChangeProof` that is returned by the `get_state_proof` API endpoint.
Each epoch change can be detected by looking at the `next_validator_set` field of the LedgerInfo states, when this field is non-empty, then an epoch change occurred and the new validator set should be the set contained in this LedgerInfo field if the LedgerInfo verifies.
Therefore it is possible to update to the latest state and latest validator set by simply having all the LedgerInfo state with a non-empty `next_validator_set` and verifying that they correctly chain until the last one that should provide us with the required validator set to verify the current LedgerInfo state.
[Image: Screenshot 2020-12-23 at 19.42.54.png]You can read more about epoch and reconfiguration in [this tech paper](http://documentation/tech-papers/lbft-verification/lbft-verification.pdf).

# Verifying client specification

The Diem network is currently composed of Validators that achieve a consensus over the state of the blockchain. Validators will then in turn broadcast the state of the blockchain along with the necessary proofs to verify it to their Validator-operated Full Node, which are the only nodes allowed to connect to Validators except for Validators themselves. 
The current way of accessing data from the blockchain is then to query one of these Validator-operated Full Nodes that replicate the state of the blockchain. 
As Diem is a decentralised cryptocurrency, clients should not blindly trust responses to their queries. Instead, clients can use a combination of cryptographic signatures and merkle trees to ensure that every response is legitimate. This is ensured as long as more than two thirds of the Validators are trusted. 

This document specifies how a client can query Full Nodes and how a client must verify their responses.

Note that if a client queries its own, trusted Full Node, over a secure channel, it can choose to avoid verifying the full node's responses as the Full Node already verifies the data it synchronises to. Verification on the client side really make sense when the client is querying a third party Full Node. We strongly recommend verifying proofs even if you are querying a Diem Association or Diem Networks Full Node.

## Client state

Clients that want to query the blockchain must be prepared to validate responses. To do this, they must be maintain the following state:

* Latest `known_version`. The latest known version is useful to query data from the server that would allow us to correctly update our state to the latest one. 
* Latest `known_epoch`. An epoch indicates a potential change of the validator set. As such, a client can only verify responses related to the client's current epoch, unless the client updates its epoch number. See “Epoch change proofs validation” above.
* Latest `known_validator_set`. A validator set is a list of public keys that can be used to verify the state of the blockchain for a specific epoch.

Furthermore, a client that wants to validate transaction proofs will need to keep in its state the following as well:

* `LedgerInfoWithSignatures`. A ledger state along with its signature that contain the root accumulator hash used to verify transaction proofs.

For this reason, a client needs to be initialised with an epoch number and its matching validator set, then it needs to query the blockchain to update its `LedgerInfoWithSignatures` and verify it. In addition, it must be able to update its state, as new epoch numbers might imply different validator set, and the latest `LedgerInfoWithSignatures` state can change multiple times per minute on the server side.

A client lagging behind (for example at epoch 3, while the current epoch is 6) will need to query and validate the latest `LedgerInfoWithSignatures` state along with the required epoch change proofs provided by the `get_state_proof` API in order to update its state to the latest epoch.

Notice that the last `LedgerInfoWithSignatures` state of an epoch contains a non-empty `next_validator_set` field that allows to update the validator set for the next epoch if its signatures verify. These `LedgerInfoWithSignatures` are part of the `ValidatorChangeProof` provided by the `get_state_proof` API.

## Error handling

A verifying client should always handle the error as specified by the [JSON-RPC client specification](https://github.com/diem/diem/blob/master/json-rpc/docs/client_implementation_guide.md#error-handling), but it should also have the following extra error cases:

* Failed proof verification. When a proof fails to validate, this should not be a silent error and should trigger an alert that prompts the user that something has gone awry.
* Invalid signatures. All the signatures a client receives are supposed to verify against their known signer, shall a signature fail to verify, this should not be a silent error and should trigger an alert that prompts the user that something has gone awry.

Notice that the above failures both hint at byzantine behaviour of the connected node, this can be caused by an active attempt to mislead a client into misbehaving or by a node that is actively trying to harm the network.

## Handling JSON-RPC responses

All JSON-RPC responses should be valid `serde_json::Value`, i.e. valid JSON values. These JSON responses and the HTTP responses are containing some extra data that typically allow to verify that the server you are querying is up to date. See [Diem extensions JSON-RPC spec](https://github.com/diem/diem/blob/master/json-rpc/json-rpc-spec.md#diem-extensions) and [HTTP header extensions](https://github.com/diem/diem/blob/master/json-rpc/json-rpc-spec.md#http-response-headers-extensions) for more details, notice that the later should be preferred.

All responses should be verifying that:

1. the JSON-RPC `diem_chain_id` response object and the `X-Diem-Chain-Id` HTTP Header extension are both corresponding to the network id you are querying.
2. the JSON-RPC `diem_ledger_version` response object and the `X-Diem-Ledger-Version` HTTP Header extension are both corresponding and are equal or larger to the `known_version` kept in our state.
3. the JSON-RPC `diem_ledger_timestampusec` response object and the `X-Diem-Ledger-TimestampUsec` HTTP Header extension are both corresponding and are within 5 minutes of our own timestamp.

## Getting and verifying the latest ledger state

In order to verify both events and transactions proofs, it is important for the client to always batch these calls with a `get_ledger_state` call in order to ensure it has the latest validator set and the right accumulator hash.
The `get_ledger_state` endpoint allows to get the latest state known by the server relative to the version known by the client provided as a parameter, along with the `ValidatorChangeProof` from that known version to the latest version.

Parameters: `u64: known_version`

The required verifications to ensure proper validation are:

1. Verify the JSON-RPC response as per the “Handling JSON-RPC responses” section.
2. Verify that the 
3. TKTK

## Getting and verifying Transactions

In order to get a list of transactions and their proofs, a verifying client can use the `get_transactions_with_proofs` JSON-RPC endpoint. This endpoint is described here: https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_transactions_with_proofs.md

When calling this endpoint, one needs to pass as parameters a `start_version` specifying the version of the first transaction we would like to get and a `limit` specifying the maximum number of transactions to be returned. It is possible to receive less transactions than that `limit` since it is possible to query one of the last transactions known by the queried server.

You need to parse the returned data in order to extract a list `serialized_transactions` which contains the raw transactions as BCS encoded hex strings, and a list of `TransactionsProofs` that contain the `ledger_info_to_transaction_infos_proof`, which is a `Vec<AccumulatorRangeProof<TransactionAccumulatorHasher>>` encoded in BCS and represented as a hex string, and the `transaction_infos` which is a `Vec<TransactionInfo>` encoded in BCS and represented as a hex string.

In order to verify the returned transactions, you need to first:

1. Verify the JSON-RPC response as per the “Handling JSON-RPC responses” section.
2. Verify that the size of `serialized_transactions`, the decoded `ledger_info_to_transaction_infos_proof` and the decoded `transaction_infos` all match.
3. TKTK

## Getting and verifying Events

In order to get a list of events, including events that could refer to invalid transactions that didn’t make it to the ledger state, on can use the `get_events_with_proofs` JSON-RPC endpoint. This endpoint is described here: https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_events_with_proofs.md

The required verifications are:

1. Verify the JSON-RPC response as per the “Handling JSON-RPC responses” section.
2. TKTK

## Getting and verifying Accounts

TKTK

The required verifications are:

1. Verify the JSON-RPC response as per the “Handling JSON-RPC responses” section.
2. TKTK

## What’s missing from this specification

This spec is not self-contained and we are not defining the following items:

* BCS serialization 
* event streams and event keys
* the underlying cryptography and the methods to verify a AccumulatorRangeProof for instance.


