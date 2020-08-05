## Method get_account

**Description**

Get the latest account information for a given account address.


### Parameters

| Name    | Type   | Description                 |
|---------|--------|-----------------------------|
| account | string | Hex-encoded account address |


### Returns

[Account](type_account.md) - If account exists

Null - If account does not exist


### Example

```
// Request: fetches account for account address "1668f6be25668c1a17cd8caf6b8d2f25"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account","params":["1668f6be25668c1a17cd8caf6b8d2f25"],"id":1}' https://client.testnet.libra.org/v1

// Response
{
  "id": 1,
  "jsonrpc": "2.0",
  "libra_chain_id": 2,
  "libra_ledger_timestampusec": 1596694123535425,
  "libra_ledger_version": 3307422,
  "result": {
    "authentication_key": "d939b0214b484bf4d71d08d0247b755a1668f6be25668c1a17cd8caf6b8d2f25",
    "balances": [
      {
        "amount": 12070000000,
        "currency": "LBR"
      }
    ],
    "delegated_key_rotation_capability": false,
    "delegated_withdrawal_capability": false,
    "is_frozen": false,
    "received_events_key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
    "role": {
      "parent_vasp": {
        "base_url": "https://libra.org",
        "compliance_key": "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
        "expiration_time": 18446744073709552000,
        "human_name": "testnet",
        "num_children": 0
      }
    },
    "sent_events_key": "01000000000000001668f6be25668c1a17cd8caf6b8d2f25",
    "sequence_number": 50
  }
}

// Sample Response for non-existent account
{
  "id": 1,
  "jsonrpc": "2.0",
  "libra_chain_id": 2,
  "libra_ledger_timestampusec": 1596694171246702,
  "libra_ledger_version": 3307614,
  "result": null
}
```
