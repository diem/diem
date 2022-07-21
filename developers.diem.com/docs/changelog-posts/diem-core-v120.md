---
title: "Release v1.2 Announcement"
slug: "diem-core-v120"

hidden: false
---
We're announcing the new release v1.2.0

Hi all,

We're announcing the new release v1.2.0.
Release `diem-core-v1.2.0` will be deployed to Diem Testnet on **04/28/2021**, and the `diem-framework-v1.2.0` will be enabled on **04/29/2021**.

Feel free to post your questions in this [discussion thread](https://developers.diem.com/main/discuss/60875bd734f240002a026c92).

[diem-core-v1.2.0](https://github.com/diem/diem/tree/diem-core-v1.2.0)

[Full changelog since diem-core-v1.1.0](https://github.com/diem/diem/compare/diem-core-v1.1.0...diem-core-v1.2.0)

## Noticeable changes

**Fixed bugs:**

- \[Bug\] Broken image in storage README [\#6934](https://github.com/diem/diem/issues/6934)
- \[Bug\] json-rpc method call get\_account\_state\_with\_proof returns server internal error when given an old version [\#6872](https://github.com/diem/diem/issues/6872)
- \[Bug\] Documentation for rotate keys is wrongly using Public Key where it should be Authentication Key [\#7702](https://github.com/diem/diem/issues/7702)
- \[mempool\] Perf regression with CT burst mode [\#7577](https://github.com/diem/diem/issues/7577)
- \[Bug\] account can't send any txn if its balance is less than 600 [\#7238](https://github.com/diem/diem/issues/7238)
- \[Bug\] The url is missing in the sample code curl [\#6887](https://github.com/diem/diem/issues/6887)
- \[Bug\] "Unable to get currency info for Coin1 when converting to on-chain" [\#7368](https://github.com/diem/diem/issues/7368)


**Notable changes:**

**[Build]**

- [rust] upgrade to rust 1.51 [\#8017](https://github.com/diem/diem/pull/8017)

**[Consensus]**

- [consensus] Add reputation window proposals/vote metrics [\#7758](https://github.com/diem/diem/pull/7758)
- [consensus] avoid mempool long poll if root block has user txns [\#7182](https://github.com/diem/diem/pull/7182)

**[Diem Framework]**

- [diem framework] Limit the number of validators in the network to 256 or less [\#6952](https://github.com/diem/diem/pull/6952)
- [language] follow up to latest prologue/epilogue changes [\#7063](https://github.com/diem/diem/pull/7063)
- [diem-types] Update currency code restrictions now that we have the real codes. [\#7252](https://github.com/diem/diem/pull/7252)
- [diem framework] make script allowlist immutable [\#7556](https://github.com/diem/diem/pull/7556)
- [diem framework] Add support for concurrent preburns to the DF [\#7648](https://github.com/diem/diem/pull/7648)
- [diem framework] delete tiered minting logic [\#7920](https://github.com/diem/diem/pull/7920)
- [diem framework] Increase max number of child accounts to 2^16 form 2^8 [\#6955](https://github.com/diem/diem/pull/6955)
- [diem framework] introduce diem-framework-releases [\#8028](https://github.com/diem/diem/pull/8028)
- [diem framework] Add gas constant setters  [\#8080](https://github.com/diem/diem/pull/8080)

**[JSON RPC]**

- [json-rpc] Update failing integration test by increasing timeout. [\#6942](https://github.com/diem/diem/pull/6942)
- [json-rpc] Add `preburn_queues` field to DesignatedDealer view [\#7852](https://github.com/diem/diem/pull/7852)
- [json-rpc] Adding a get_events_with_proof API [\#6539](https://github.com/diem/diem/pull/6539)
- [json-rpc] allow no params field in JSON-RPC API call request [\#7275](https://github.com/diem/diem/pull/7275)
- [json-rpc] fixing race in test with batch call [\#7001](https://github.com/diem/diem/pull/7001)
- [json-rpc] health-check for checking latest ledger info timestamp [\#7419](https://github.com/diem/diem/pull/7419)
- [json-rpc] Support TLS on JSON-RPC port [\#7297](https://github.com/diem/diem/pull/7297)
- [secure] Include Move abort code explanation in the VMStatusView [\#7123](https://github.com/diem/diem/pull/7123)

**[Logging]**

- crash-handler: wait till logs have been flushed to exit [\#7378](https://github.com/diem/diem/pull/7378)
- [diem-trace] restore previous behavior of set_diem_trace [\#7667](https://github.com/diem/diem/pull/7667)
- [logging] Add documentation to all structs and macros [\#7707](https://github.com/diem/diem/pull/7707)

**[Mempool]**

- [mempool] Add ConnectionMetadata to PeerSyncState [\#7637](https://github.com/diem/diem/pull/7637)
- [mempool] Add in configurable upstream failovers [\#7858](https://github.com/diem/diem/pull/7858)
- [mempool] Add SharedMempool GCing for non-validator peers [\#7687](https://github.com/diem/diem/pull/7687)
- [mempool] Give ordering to upstream peers [\#7809](https://github.com/diem/diem/pull/7809)
- [mempool] Remove legacy failover logic [\#7690](https://github.com/diem/diem/pull/7690)
- [mempool] return accepted for submitting same transaction in mempool [\#7174](https://github.com/diem/diem/pull/7174)
- [mempool] Split up test bootstrap network [\#7765](https://github.com/diem/diem/pull/7765)
- [mempool] Use proper NodeConfig defaults for node types [\#7689](https://github.com/diem/diem/pull/7689)

**[Move]**

- Updated to version 1.2 of Move: see the [Move release notes](https://github.com/diem/diem/blob/main/language/RELEASES.md#move-version-12) for details
- With Diem Framework version 2, transaction scripts take an owned `signer` instead of a `&signer` reference. When the on-chain Diem Framework version is set to 1, the VM still expects a `signer` reference [\#8029](https://github.com/diem/diem/pull/8029)

**[Network]**

- [network] Prioritize Peer connections and remove 2nd Public network interface [\#7927](https://github.com/diem/diem/pull/7927)
- [network] Allow Conn Manager to be used with Public interfaces [\#7925](https://github.com/diem/diem/pull/7925)
- [network] Decrease network allowed rate limits [\#7177](https://github.com/diem/diem/pull/7177)
- [network] Metric if network identity doesn't match onchain [\#7217](https://github.com/diem/diem/pull/7217)
- [rate-limiter] Allow for disabling throttle in config [\#7176](https://github.com/diem/diem/pull/7176)
- [config] Make rate limiter optional for Validators [\#6941](https://github.com/diem/diem/pull/6941)
- [diem-node] Remove shutdown method after fixing network panic [\#7231](https://github.com/diem/diem/pull/7231)

**[SDK]**

- sdk: introduce a new diem-client intended to be apart of the rust-sdk [\#7420](https://github.com/diem/diem/pull/7420)
- add CoinTradeMetadata [\#7582](https://github.com/diem/diem/pull/7582)
- Add constructor and getters for GeneralMetadataV0 as inner fields are private [\#7039](https://github.com/diem/diem/pull/7039)
- add RefundMetadata struct for generating types [\#7502](https://github.com/diem/diem/pull/7502)

**[State Sync]**

- [State Sync] Fix potential storage race condition with consensus. [\#7365](https://github.com/diem/diem/pull/7365)
- [State Sync] Make inter-component communication timeouts configurable. [\#7657](https://github.com/diem/diem/pull/7657)
- [State Sync] Remove unneccessary panics to avoid DOS attacks. [\#7778](https://github.com/diem/diem/pull/7778)
- [State Sync] Small cleanups, renames and refactors. [\#7017](https://github.com/diem/diem/pull/7017)
- [State Sync] Update state sync to prioritize preferred peers. [\#7902](https://github.com/diem/diem/pull/7902)
- [State Sync] Perform basic chunk request verification before processing. [\#7520](https://github.com/diem/diem/pull/7520)
- [State Sync] Use checked arithmetic operations to avoid over/under-flows [\#7065](https://github.com/diem/diem/pull/7065)
- [State Sync] Use optimistic fetch for both waypoint and verifiable chunk processing. [\#7595](https://github.com/diem/diem/pull/7595)
- [state-sync] removing integer overflow [\#7060](https://github.com/diem/diem/pull/7060)

**[Storage]**

- [backup] metadata cache defaults to temp dir [\#7940](https://github.com/diem/diem/pull/7940)
- [backup] support for specifying trusted waypoints in restore / verify [\#7329](https://github.com/diem/diem/pull/7329)
- [diemdb_bench] initial_commit [\#7407](https://github.com/diem/diem/pull/7407)
- [storage] expose RbReader::get_last_version_before_timestamp() [\#7740](https://github.com/diem/diem/pull/7740)
- [storage] smaller prune_window [\#7197](https://github.com/diem/diem/pull/7197)
- [Proof] Make SparseMerkleProof generic [\#7363](https://github.com/diem/diem/pull/7363)

**[TCB]**

- [safety-rules] cleaning [\#7680](https://github.com/diem/diem/pull/7680)
- [safety-rules] removing redundant safety data update [\#7684](https://github.com/diem/diem/pull/7684)
- [Secure Storage] Reduce default timeouts for vault and support custom op tooling timeouts. [\#7743](https://github.com/diem/diem/pull/7743)
- [Secure Storage] Update vault request timeout and error handling. [\#7193](https://github.com/diem/diem/pull/7193)
- [Secure Storage] Vault timeout workaround due to ureq bug. [\#7828](https://github.com/diem/diem/pull/7828)
- [secure-storage] support github branches [\#7150](https://github.com/diem/diem/pull/7150)
- [tcb][safety_rules] added state machine specification [\#7842](https://github.com/diem/diem/pull/7842)
- [Vault Storage] Make the vault client support configurable timeout values [\#7675](https://github.com/diem/diem/pull/7675)