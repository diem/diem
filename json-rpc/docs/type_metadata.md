## Type Metadata



| Name                       | Type           | Description                                    |
|----------------------------|----------------|------------------------------------------------|
| version                    | unsigned int64 | The latest block (ledger) version              |
| timestamp                  | unsigned int64 | The latest block (ledger) timestamp, unit is microsecond |
| chain_id                   | unsigned int8  | Chain ID of the Libra network                  |
| script_hash_allow_list     | List<string>   | List of allowed scripts hex-encoded hash bytes, server may not return this field if the allow list not found in on chain configuration. |
| module_publishing_allowed  | boolean        | True for allowing publishing customized script, server may not return this field if the flag not found in on chain configuration. |
| libra_version              | unsigned int64 | Libra chain major version number              |
| accumulator_root_hash      | string         | accumulator root hash of the block (ledger) version |
| dual_attestation_limit     | unsigned int64 | The dual attestation limit on-chain. Defined in terms of micro-LBR. |

Note:
1. see [LibraTransactionPublishingOption](../../language/stdlib/modules/doc/LibraTransactionPublishingOption.md) for more details of `script_hash_allow_list` and `module_publishing_allowed`.
2. Fields `script_hash_allow_list`, `module_publishing_allowed` and `libra_version` are only returned when requesting latest version by [get_metadata](method_get_metadata.md) method call.


### Example


```
// Request: fetches current block metadata
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_metadata","params":[],"id":1}' https://testnet.libra.org/v1

// Response
{
  "id": 1,
  "jsonrpc": "2.0",
  "libra_chain_id": 2,
  "libra_ledger_timestampusec": 1596680521771648,
  "libra_ledger_version": 3253133,
  "result": {
    "timestamp": 1596680521771648,
    "version": 3253133,
    "chain_id": 4,
    "script_hash_allow_list": [
        <allowed scripts hex-encoded hash string>
    ],
    "module_publishing_allowed": false,
    "libra_version": 1,
    "accumulator_root_hash": "<hash string>",
    "dual_attestation_limit": 1000000000
  }
}
```
