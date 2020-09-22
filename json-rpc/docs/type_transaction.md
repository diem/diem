## Type Transaction

**Description**

A transaction on the blockchain.


### Attributes


| Name                      | Type                                     | Description                                                                                |
|---------------------------|------------------------------------------|--------------------------------------------------------------------------------------------|
| version                   | unsigned int64                           | The on-chain version or unique identifier of this transaction                              |
| transaction               | [TransactionData](#type-transactiondata) | Transaction payload |
| hash                      | string                                   | hex encoded string of hash of this transaction, hex encoded |
| bytes                     | string                                   | hex encoded string of raw bytes of the transaction, hex encoded |
| events                    | List<[Event](type_event.md)>             | List of associated events. Empty for no events                                             |
| vm_status                 | [VMStatus](#type-vmstatus)               | The returned status of the transaction after being processed by the VM                     |
| gas_used                  | unsigned int64 | Amount of gas used by this transaction, to know how much you paid for the transaction, you need multiply it with your RawTransaction#gas_unit_price |

Note:
* For the gas_used, internally within the VM we scale the gas units down by 1000 in order to allow granularity of costing for instruction, but without having to use floating point numbers, but we do round-up the gas used to the nearest "1" when we convert back out.
* Formula to create hash for a signed transaction before it is executed: hex-encode(sha3-256([]byte("LIBRA::Transaction")) + []byte(0) + signed transaction bytes) ([implementation example](https://github.com/libra/libra-client-sdk-go/blob/master/libratypes/hash.go#L27))


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
| type                | string         | Type of TransactionData                                         |

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
| max_gas_amount            | unsigned int64         | Maximum amount of gas that can be spent for this transaction          |
| gas_unit_price            | unsigned int64         | Maximum gas price to be paid per unit of gas                          |
| gas_currency              | string                 | Gas price currency code                                               |
| expiration_timestamp_secs | unsigned int64         | The expiration time (Unix Epoch in seconds) for this transaction      |
| script_hash               | string                 | Hex-encoded hash of the script used in this transaction               |
| script_bytes              | string                 | Hex-encoded string of the bytes of the script                         |
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

| Name                      | Type           | Description                                                                                             |
|---------------------------|----------------|---------------------------------------------------------------------------------------------------------|
| type                      | string         | constant of string "peer_to_peer_transaction"                                                           |
| receiver                  | string         | Hex-encoded account address of the receiver                                                             |
| amount                    | unsigned 64    | amount transfered in microlibras                                                                        |
| currency                  | string         | Currency code                                                                                           |
| metadata                  | string         | Metadata from RawTransaction#Script, LCS serialized hex-encoded string [server side data structure][1]. |
| metadata_signature        | string         | Hex-encoded metadata signature, use this to validate metadata                                           |

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

A `VMStatus` is an object of one of following type:

#### executed

Successful execution.

```
{type: "executed"}
```

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "executed"                              |

#### out_of_gas
Transaction execution runs out of gas, no effect.

```
{type: "out_of_gas"}
```

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "out_of_gas"                              |

#### move_abort

Object representing the abort condition raised by Move code via `abort` or `assert` during execution of a transaction by the VM on the blockchain.

```
{ type: "move_abort", location: string, abort_code: unsigned int64 }
```


| Name       | Type           | Description                                                                                                                                  |
|------------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| type       | string         | constant of string "move_abort"                              |
| location   | string         | String of the form "address::moduleName" where the abort condition was triggered. "Script" if the abort was raised in the transaction script |
| abort_code | unsigned int64 | Abort code raised by the Move module                                                                                                         |

#### execution_failure

Object representing execution failure while executing Move code, but not raised via a Move abort (e.g. division by zero) during execution of a transaction by the VM on the blockchain.

```
{ type: "execution_failure", location: string, function_index: unsigned int16, code_offset: unsigned int16 }
```


| Name           | Type           | Description                                                                                                                                  |
|----------------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| type           | string         | constant of string "execution_failure" |
| location       | string         | String of the form "address::moduleName" where the execution error occurred. "Script" if the execution error occurred while execution code that was part of the transaction script. |
| function_index | unsigned int16 | The function index in the `location` where the error occurred |
| code_offset    | unsigned int16 | The code offset in the function at `function_index` where the execution failure happened |

#### miscellaneous_error
A general error indicating something went wrong with the transaction outside of it's execution. This could include but is not limited to

* An error caused by the script/module, possibly:
  * A bytecode verification error
  * A failure to deserialize the transaction
* An error caused by the transaction arguments, possibly:
  * A failure to deserialize the arguments for the given type
  * An argument's type is not valid for the given script

Note that this explicitly excludes any invariant violation coming from inside of the VM. A transaction that hits any such invariant violation error will be discarded.

```
{type: "miscellaneous_error"}
```

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| type                      | string         | constant of string "miscellaneous_error"                              |

[1]: https://libra.github.io/libra/libra_types/transaction/metadata/enum.Metadata.html "Transaction Metadata"
