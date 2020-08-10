## Method get_metadata

**Description**

Get the blockchain metadata (e.g., state as known to the current full node).

### Parameters

| Name    | Type           | Description                                                                                          |
|---------|----------------|------------------------------------------------------------------------------------------------------|
| version | unsigned int64 | The transaction version, this parameter is optional, default is server's latest transaction version. |


### Returns

[Metadata](type_metadata.md)

### Example

```
// Request: fetches current block metadata
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_metadata","params":[],"id":1}' https://client.testnet.libra.org/v1

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
  }
}
```
