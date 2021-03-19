## Method get_account

**Description**

Get the latest account information for a given account address.


### Parameters

| Name    | Type           | Description                                                                                         |
|---------|----------------|-----------------------------------------------------------------------------------------------------|
| account | string         | Hex-encoded account address                                                                         |
| version | unsigned int64 | The transaction version, this parameter is optional, default is server's latest transaction version |

> Depending on server's configuration, querying too old version may get error indicating data is pruned.


### Returns

[Account](type_account.md) - If account exists

Null - If account does not exist


### Example

```
// Request: fetches account for account address "1668f6be25668c1a17cd8caf6b8d2f25"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account","params":["1668f6be25668c1a17cd8caf6b8d2f25"],"id":1}' https://testnet.diem.com/v1

// Response
{
   "diem_chain_id" : 2,
   "jsonrpc" : "2.0",
   "diem_ledger_timestampusec" : 1597084681499780,
   "result" : {
      "delegated_key_rotation_capability" : false,
      "received_events_key" : "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "authentication_key" : "d939b0214b484bf4d71d08d0247b755a1668f6be25668c1a17cd8caf6b8d2f25",
      "balances" : [
         {
            "amount" : 2194000000,
            "currency" : "XDX"
         }
      ],
      "sequence_number" : 11,
      "delegated_withdrawal_capability" : false,
      "sent_events_key" : "01000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "is_frozen" : false,
      "role" : {
        "type": parent_vasp",
        "num_children" : 0,
        "base_url" : "https://diem.com",
        "human_name" : "testnet",
        "compliance_key" : "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
        "expiration_time" : 18446744073709551615
      }
   },
   "id" : 1,
   "diem_ledger_version" : 1303433
}

// Sample Response for non-existent account
{
  "id": 1,
  "jsonrpc": "2.0",
  "diem_chain_id": 2,
  "diem_ledger_timestampusec": 1596694171246702,
  "diem_ledger_version": 3307614,
  "result": null
}
```
