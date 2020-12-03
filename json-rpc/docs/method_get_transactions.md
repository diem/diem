## Method get_transactions

**Description**

Get the transactions on the blockchain.


### Parameters

| Name           | Type           | Description                                                          |
|----------------|----------------|----------------------------------------------------------------------|
| start_version  | unsigned int64 | Start on this transaction version for this query                     |
| limit          | unsigned int64 | Limit the number of transactions returned, the max value is 1000     |
| include_events | boolean        | Set to true, to also fetch [events](type_event.md) for each transaction |

### Returns

Array of [Transaction](type_transaction.md) objects

if include_events is false, the [events](type_event.md) field in the Transaction object will be an empty array.


### Example


```
// Request: fetches 10 transactions since version 100000
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_transactions","params":[100000, 10, false],"id":1}' https://testnet.diem.com/v1

// Response
{
  "id": 1,
  "jsonrpc": "2.0",
  "diem_chain_id": 2,
  "diem_ledger_timestampusec": 1596694433936687,
  "diem_ledger_version": 3308663,
  "result": [
    {
      "events": [],
      "gas_used": 100000000,
      "hash": "1dfa3ff87be2aa21e990324a4b70d300621f81bfb48621a74136a23c4a68b7a7",
      "transaction": {
        "timestamp_usecs": 1596085070813591,
        "type": "blockmetadata"
      },
      "version": 100000,
      "vm_status": "executed"
    },
    {
      "events": [],
      "gas_used": 100000000,
      "hash": "c1e1e8316eae3bbc0f13f8d9616591edd83adfb6fa7c8fcfbc51398e770114ef",
      "transaction": {
        "timestamp_usecs": 1596085071054623,
        "type": "blockmetadata"
      },
      "version": 100001,
      "vm_status": "executed"
    },
    {
      "events": [],
      "gas_used": 175,
      "hash": "4b31f147fd659ca68fa1382d700901e0b987d2358b421132b7a7bb2ac4542fce",
      "transaction": {
        "chain_id": 2,
        "expiration_timestamp_secs": 1596085170,
        "gas_currency": "XDX",
        "gas_unit_price": 0,
        "max_gas_amount": 1000000,
        "public_key": "86f38df4199842f3bfe0dcd003aaf739f9cddd2845b2c5a3318b1878d56b0eb8",
        "script": {
          "amount": 100000,
          "currency": "XDX",
          "metadata": "",
          "metadata_signature": "",
          "receiver": "2ce4a93c05ba7ac8e6e94736731b3ddd",
          "type": "peer_to_peer_transaction"
        },
        "script_hash": "61749d43d8f10940be6944df85ddf13f0f8fb830269c601f481cc5ee3de731c8",
        "sender": "c778574753e789661d8223bc1940fae3",
        "sequence_number": 16,
        "signature": "b66d72e3f20034b03d805c3652062a5921aabd3d01f08f9977f08ae5fc7c77694e852cfbe8e9c53c2bb1d4d268042892bf480481cf330713694777f1226fa208",
        "signature_scheme": "Scheme::Ed25519",
        "type": "user"
      },
      "version": 100002,
      "vm_status": { "type": "executed" }
    },
    ....
  ]
}
```
