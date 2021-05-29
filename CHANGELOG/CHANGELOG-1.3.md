# Changelog

## [Unreleased](https://github.com/diem/diem/tree/HEAD)

[Full Changelog](https://github.com/diem/diem/compare/diem-core-v1.2.0...HEAD)

**Fixed bugs:**

- \[cli\] Fix the logic for fetching diem_root account in the cli. [\#8136](https://github.com/diem/diem/pull/8136)
- \[Bug\] VFN crashes after 1.1 -\> 1.2 upgrade [\#8140](https://github.com/diem/diem/issues/8140)


**Notable changes:**

**[Build]**
- rust: update rust to v1.52.1 [\#8393](https://github.com/diem/diem/pull/8393)

**[Crypto]**
- \[HKDF\] do not allow \< 16 bytes seeds to prevent misuse. [\#8159](https://github.com/diem/diem/pull/8159)

**[Diem Framework]**
- Create DiemIdDomainManager on genesis [\#8412](https://github.com/diem/diem/pull/8412)
- Add move resources and scripts for DiemID [\#8336](https://github.com/diem/diem/pull/8336)
  - [Related to DIP-10](https://github.com/diem/dip/blob/main/dips/dip-10.md) specifically the [On-Chain Data section](https://github.com/diem/dip/blob/main/dips/dip-10.md#on-chain-data) of that DIP.
- \[diem_vm\] add support for multi agent transactions[\#7427](https://github.com/diem/diem/pull/7427)
   - [Related to DIP-169](https://github.com/diem/dip/blob/main/dips/dip-169.md)
- \[transaction-replay\] Implement an api for fetching annotated event data [\#8276](https://github.com/diem/diem/pull/8276)
- \[writeset-generator\] Create the writeset patch for diem-framework-1.2.0-rc0 release [\#8082](https://github.com/diem/diem/pull/8082)
- \[diem framework\] remove DiemAccount::destroy\_signer + associated native function [\#8163](https://github.com/diem/diem/pull/8163)
- \[writeset-generator\] Fix a few minor issues for writeset generator tool [\#8446](https://github.com/diem/diem/pull/8446)

**[JSON RPC/Client]**
- String type for DiemID [\#8362](https://github.com/diem/diem/pull/8362) ([sunmilee](https://github.com/sunmilee))
- \[json-rpc\]: remove the json field metadata if the value is null [\#8319](https://github.com/diem/diem/pull/8319)
- \[JSONRPC\] Separate data fetching from RPC Server implementation [\#8278](https://github.com/diem/diem/pull/8278)
- json-rpc: stronger typed RPCs [\#8261](https://github.com/diem/diem/pull/8261)
- \[json-rpc\]: add raw BCS bytes to unknown event data [\#8241](https://github.com/diem/diem/pull/8241)

**[Mempool]**
- \[mempool\] Fix upstream peers counter [\#8245](https://github.com/diem/diem/pull/8245)

**[Move]**

- Updated to version 1.3 of Move: see the Move release notes for details
- [diem-vm] Implement prefetching logic for account blobs [\#8248](https://github.com/diem/diem/pull/8248)

**[Network]**
- \[network\] Remove rate-limit key from metrics [\#8142](https://github.com/diem/diem/pull/8142)

**[SDK]**
- \[sdk\] Add get_transactions to verifying client. Extract common conversions to json-rpc-types. [\#8398](https://github.com/diem/diem/pull/8398)
- \[sdk\] Add move error explanations to transaction builder [\#8410](https://github.com/diem/diem/pull/8410)
- \[sdk\] Add get_events to json-rpc verifying client [\#8408](https://github.com/diem/diem/pull/8408)
- [sdk] Add get_currencies and get_network_status to verifying client [\#8416](https://github.com/diem/diem/pull/8416)
- sdk: release v0.0.2 [\#8367](https://github.com/diem/diem/pull/8367) ([bmwill](https://github.com/bmwill))
- sdk: introduce offchain api crate [\#7700](https://github.com/diem/diem/pull/7700)
- \[sdk\] Add verifying and non-verifying client equivalence testing [\#8368](https://github.com/diem/diem/pull/8368)
- \[sdk\] Add json-rpc VerifyingClient [\#8231](https://github.com/diem/diem/pull/8231)
- \[sdk\] experimental APIs for fetching deserialized Move events, resour… [\#8230](https://github.com/diem/diem/pull/8230)
- sdk: teach TransactionFactory to create script function transactions [\#8150](https://github.com/diem/diem/pull/8150)
- sdk: use correct response types in experimental proof API's [\#8207](https://github.com/diem/diem/pull/8207)
- \[sdk\] use correct response types in experimental proof API's [\#8206](https://github.com/diem/diem/pull/8206)
- faucet: remove race condition when getting sequence numbers [\#8310](https://github.com/diem/diem/pull/8310)
- client: use stronger typed EventKey for requests [\#8333](https://github.com/diem/diem/pull/8333)
- client: don't retry submit requests [\#8266](https://github.com/diem/diem/pull/8266)

**[State Sync]**
- \[State Sync Performance Test\] Add additional checks for possible error cases. [\#8365](https://github.com/diem/diem/pull/8365)

**[Storage]**
- \[scratchpad\] Criterion benchmarks [\#8359](https://github.com/diem/diem/pull/8359) [\#8388](https://github.com/diem/diem/pull/8388)
- \[pruner\] fix busy looping after db-restore [\#8312](https://github.com/diem/diem/pull/8312)
- \[storage\] delete unimplemented multi\_get interface [\#8254](https://github.com/diem/diem/pull/8254)
- \[smt\] experimental batch update [\#8119](https://github.com/diem/diem/pull/8119)
- Sparse Merkle Tree batch update [\#8178](https://github.com/diem/diem/pull/8178) [\#8215](https://github.com/diem/diem/pull/8215)[\#8448](https://github.com/diem/diem/pull/8448)

**[Operational Tool]**
- \[diem-operational-tool\] Add x25519 Key generation and peer info [\#8369](https://github.com/diem/diem/pull/8369)
- \[operational-tool\] Simplify network checker [\#8348](https://github.com/diem/diem/pull/8348)
- \[Operational Tool\] Make backend arguments optional for non-required commands [\#8308](https://github.com/diem/diem/pull/8308)

**[Specification/Doc]**
- Add Payment Metadata [\#8335](https://github.com/diem/diem/pull/8335)
- ping command spec [\#8121](https://github.com/diem/diem/pull/8121)
- travel rule spec [\#8135](https://github.com/diem/diem/pull/8135)


\* *This Changelog was partially generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
