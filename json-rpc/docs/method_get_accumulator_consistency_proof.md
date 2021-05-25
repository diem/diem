## Method get_accumulator_consistency_proof

**Description**

Gets an accumulator consistency proof starting from `client_known_version` (or pre-genesis if null or not present) until `ledger_version` (or the server's current version if null or not present).

In other words, if the client has an accumulator summary for `client_known_version`, they can use the result from this API to efficiently extend their accumulator to `ledger_version` and prove that the new accumulator is consistent with their old accumulator. By consistent, we mean that by appending the actual `ledger_version - client_known_version` transactions to the old accumulator summary you get the new accumulator summary.

If the client is starting up for the first time and has no accumulator summary yet, they can call this without `client_known_version`, i.e., pre-genesis, to get the complete accumulator summary up to `ledger_version`.

### Parameters

| Name                 | Type                   | Description                                                                                                                                                                                                     |
|----------------------|------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| client_known_version | Option<unsigned int64> | The version of the client's current accumulator summary. This parameter is optional; if null or not present, the proof will start from pre-genesis, allowing the client to build an initial accumulator summary |
| ledger_version       | Option<unsinged int64> | The target version of the consistency proof. The parameter is optional; if null or not present, the target version will be the server's current ledger version.                                                 |

### Returns

Returns an object with a field `ledger_consistency_proof`. The field is a hex-encoded string of the raw BCS bytes of the `AccumulatorConsistencyProof` type, spanning versions `client_known_version` to `ledger_version`.

Example JSON-RPC response:
```json
{
    "id": 1,
    "jsonrpc": "2.0",
    "diem_chain_id": 4,
    "diem_ledger_timestampusec": 1621975989145066,
    "diem_ledger_version": 148,
    "result": {
        "ledger_consistency_proof": "04207311cfe9202a5ece145829ef74dfe6f2bb835ce9f747a94437360bfb6d1d96e2200134b4b5f232395c5553af9b99558d33b11e338cc0d62606a0922fe1d2d0e0582061e9761b07c1223b11ac93a70c0a16833a80e8f6f2ae5d285cda6f592013712f20d7907576215fe0c824f92d03d12fda9a05069e8c7fdd1a6eaeb74922b2251f0d",
    },
}
```
