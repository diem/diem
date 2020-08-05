## Type Transaction

**Description**

A transaction on the blockchain.


### Attributes


| Name                      | Type                                     | Description                                                                                |
|---------------------------|------------------------------------------|--------------------------------------------------------------------------------------------|
| version                   | unsigned int64                           | The on-chain version or unique identifier of this transaction                              |
| events                    | List<[Event](type_event.md)>             | List of associated events. Empty for no events                                             |
| transaction_data          | [TransactionData](#type-transactiondata) | Metadata for this transaction                                                              |
| vm_status                 | [VMStatus|(#type-vmstatus)               | The returned status of the transaction after being processed by the VM                     |
| gas_used                  | unsigned int64 | Amount of gas used by this transaction, to know how much you paid for the transaction, you need multiply it with your RawTransaction#gas_unit_price |

Note: For the gas_used, internally within the VM we scale the gas units down by 1000 in order to allow granularity of costing for instruction, but without having to use floating point numbers, but we do round-up the gas used to the nearest "1" when we convert back out.


### Example

```
    {
      "version": 100,
      "transaction_data": {
         "type": "user"
         "sender": "0xc94770007dda54cF92009BFF0dE90c06F603a09f",
         "sequence_number": 0,
         "max_gas_amount": 7000,
         "gas_unit_price": 3,
         "expiration_timestamp_secs": 1582007787665718,
      },
      "events": [] // empty because include_events is set to false
    }
```

### Type TransactionData

Transaction data is serialized into one JSON object with a "type" field to indicate it's type.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | Type name of EventData                                         |

#### blockmetadata

A Libra network transaction that contains the metadata for the block. This transaction is always at the beginning of a block.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | constant of string "blockmetadata"                             |
| timestamp_usecs     | unsigned int64 | Timestamp for the current block, in microseconds               |

#### writeset

A Libra network transaction that modifies storage data directly. Currently, no details are exposed in the API.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | constant of string "writeset"                                  |

#### user

User submitted transaction.

| Name                      | Type                   | Description                                                           |
|---------------------------|------------------------|-----------------------------------------------------------------------|
| type                      | string                 | constant of string "user"                                             |
| sender                    | string                 | Hex-encoded account address of the sender                             |
| signature_scheme          | string                 | Signature scheme used to sign this transaction                        |
| signature                 | string                 | Hex-encoded signature of this transaction                             |
| public_key                | string                 | Hex-encoded public key of the transaction sender                      |
| sequence_number           | unsigned int64         | Sequence number of this transaction corresponding to sender's account |
| chain_id                  | unsigned int8          | Chain ID of the Libra network this transaction is intended for        |
| max_gas_ammount           | unsigned int64         | Maximum amount of gas that can be spent for this transaction          |
| gas_unit_price            | unsigned int64         | Maximum gas price to be paid per unit of gas                          |
| expiration_timestamp_secs | unsigned int64         | The expiration time (Unix Epoch in seconds) for this transaction      |
| script_hash               | string                 | Hex-encoded hash of the script used in this transaction               |
| script                    | [Script](#type-script) | The transaction script and arguments of this transaction              |

TODO: how the script_hash is created?

#### unknown

Metadata for unsupported transaction types

| Name                      | Type                   | Description                                                           |
|---------------------------|------------------------|-----------------------------------------------------------------------|
| type                      | string                 | constant of string "unknown"                                          |


### Type Script

The transaction script and arguments of this transaction.

Transaction data is serialized into one JSON object with a "type" field to indicate it's type.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | Type name of Script data                                       |


**All other fields are arguments of the Move script submitted with RawTransaction#Script.**


#### peer_to_peer_transaction

Transaction script for peer-to-peer transfer of resource

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "peer_to_peer_transaction"                         |
| receiver                  | string         | Hex-encoded account address of the receiver                           |
| auth_key_prefix           | string         | Hex-encoded auth_key prefix                                           |
| signature                 | string         | Hex-encoded signature of this transaction                             |
| public_key                | string         | Hex-encoded public key of the transaction sender                      |
| sequence_number           | unsigned int64 | Sequence number of this transaction corresponding to sender's account |
| metadata                  | string         | Metadata from RawTransaction#Script, LCS serialized hex-encoded string [server side data structure](https://github.com/libra/libra/blob/1edf9de5749b9ae54202ef07b2b666bfe4f125d2/types/src/transaction/metadata.rs#L13). TODO: update doc after |

#### mint_transaction

Transaction script for a special transaction used by the faucet to mint Libra and send to a user.

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "mint_transaction"                                 |
| receiver                  | string         | Hex-encoded account address of the receiver                           |
| auth_key_prefix           | string         | Hex-encoded auth_key prefix                                           |
| amount                    | unsigned int64 | The amount of microlibras being sent                                  |


#### unknown_transaction

Currently unsupported transaction script

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "unknown_transaction"                              |


### Type VMStatus

A `VMStatus` is string value for the followings:
- `executed` indicating successful execution
- `out_of_gas` indicating transaction execution runs out of gas.
- `verification_error` Transaction verification error
- `deserialization_error` deserializating transaction failed

The following 2 VMStatus will be serialized as object:
- `move_abort` indicating an `abort` ocurred inside of a Move program
- `execution_failure` indicating an transaction execution failure

#### move_abort

Object representing the abort condition raised by Move code via `abort` or `assert` during execution of a transaction by the VM on the blockchain.

```
{ "move_abort": {location: string, abort_code: unsigned int64} }
```


| Name       | Type           | Description                                                                                                                                  |
|------------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| location   | string         | String of the form "address::moduleName" where the abort condition was triggered. "Script" if the abort was raised in the transaction script |
| abort_code | unsigned int64 | Abort code raised by the Move module                                                                                                         |


#### execution_failure

Object representing execution failure while executing Move code, but not raised via a Move abort (e.g. division by zero) during execution of a transaction by the VM on the blockchain.

```
{ "execution_failure": {location: string, function_index: unsigned int16, code_offset: unsigned int16} }
```

### Attributes

| Name           | Type           | Description                                                                                                                                  |
|----------------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| location       | string         | String of the form "address::moduleName" where the execution error occurred. "Script" if the execution error occurred while execution code that was part of the transaction script. |
| function_index | unsigned int16 | The function index in the `location` where the error occurred |
| code_offset    | unsigned int16 | The code offset in the function at `function_index` where the execution failure happened |
