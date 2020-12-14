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
* Formula to create hash for a signed transaction before it is executed: hex-encode(sha3-256([]byte("DIEM::Transaction")) + []byte(0) + signed transaction bytes) ([implementation example](https://github.com/diem/client-sdk-go/blob/master/diemtypes/hash.go#L27))


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

A Diem network transaction that contains the metadata for the block. This transaction is always at the beginning of a block.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | constant of string "blockmetadata"                             |
| timestamp_usecs     | unsigned int64 | Timestamp for the current block, in microseconds               |

#### writeset

A Diem network transaction that modifies storage data directly. Currently, no details are exposed in the API.

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
| chain_id                  | unsigned int8          | Chain ID of the Diem network this transaction is intended for        |
| max_gas_amount            | unsigned int64         | Maximum amount of gas that can be spent for this transaction          |
| gas_unit_price            | unsigned int64         | Maximum gas price to be paid per unit of gas                          |
| gas_currency              | string                 | Gas price currency code                                               |
| expiration_timestamp_secs | unsigned int64         | The expiration time (Unix Epoch in seconds) for this transaction      |
| script_hash               | string                 | Hex-encoded sha3 256 hash of the script binary code bytes used in this transaction |
| script_bytes              | string                 | Hex-encoded string of BCS bytes of the script, decode it to get back transaction script arguments |
| script                    | [Script](#type-script) | The transaction script and arguments of this transaction, you can decode `script_bytes` by BCS to get same data. |

Note: script_hash is not hash of the script_bytes, it's hash of the script binary code bytes. More specifically, you can get same hash string by the following steps:

    1. Decode script_bytes into script call [struct](https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.Script.html).
    2. Sha3 256 hash of the code binary bytes in the script call struct.
    3. Hex-encode the hash result bytes.

* You can decode transaction script call ([struct](https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.Script.html)) from script_bytes by BCS deserializer.
* If script_bytes is empty, it means transaction is not a [TransactionPayload#Script](https://developers.diem.com/docs/rustdocs/diem_types/transaction/enum.TransactionPayload.html#variant.Script).
You may decode Transaction#bytes by BCS deserializer for more details.


#### unknown

Metadata for unsupported transaction types

| Name                      | Type                   | Description                                                           |
|---------------------------|------------------------|-----------------------------------------------------------------------|
| type                      | string                 | constant of string "unknown"                                          |


### Type Script

The transaction script and arguments of the script call.

| Name           | Type         | Description                                   |
|----------------|--------------|-----------------------------------------------|
| type           | string       | Name of the script code, see [transaction script doc](../../language/stdlib/transaction_scripts/doc/transaction_script_documentation.md) for all available script names. |
| code           | string       | Hex-encoded compiled move script bytes        |
| arguments      | List<string> | List of string value of the script arguments. Contains type information. |
| type_arguments | List<string> | List of type arguments, converted into string |

* Argument value to string formatting:
  * u8 value `12` => "{U8: 12}"
  * u64 value `12244` => "{U64: 12244}"
  * u128 value `12244` => "{U128: 12244}"
  * boolean value `true` => "{BOOL: true}"
  * Account address value => "{ADDRESS: <hex-encoded account address bytes>}"
  * List<u8> value => "{U8Vector:: 0x<hex-encoded bytes>}"


#### unknown

Transaction script is unknown.

* When script code can't be recognized, type will be set to `unknown`, code, arguments and type_arguments will still be provided.
* When transaction payload is not a script (see [TransactionPayload](https://developers.diem.com/docs/rustdocs/diem_types/transaction/enum.TransactionPayload.html)),
type will be set to `unknown`, code, arguments and type_arguments will not be provided.

| Name                      | Type           | Description                  |
|---------------------------|----------------|------------------------------|
| type                      | string         | constant of string "unknown" |


#### peer_to_peer_with_metadata

This type was named `peer_to_peer_transaction`, to keep script#type consistent with stdlib transaction script names, we renamed it `peer_to_peer_with_metadata`.
This is the only type we decoded script arguments and type_arguments as named fields for backward compatible:

| Name                      | Type           | Description                                                         |
|---------------------------|----------------|---------------------------------------------------------------------|
| receiver                  | string         | Hex-encoded account address of the receiver                         |
| amount                    | unsigned int64 | Amount transfered.                                                  |
| currency                  | string         | Currency code.                                                      |
| metadata                  | string         | Metadata of the transaction, BCS serialized hex-encoded string.     |
| metadata_signature        | string         | Hex-encoded metadata signature, use this to validate metadata       |

Note: for metadata and metadata_signature, see [DIP-4](https://dip.diem.com/dip-4/) for more details.

### Type VMStatus

A `VMStatus` is an object of one of following type:

#### executed

Successful execution.

```
{type: "executed"}
```

| Name                      | Type           | Description                                       |
|---------------------------|----------------|---------------------------------------------------|
| type                      | string         | constant of string "executed"                     |

#### out_of_gas
Transaction execution runs out of gas, no effect.

```
{type: "out_of_gas"}
```

| Name                      | Type           | Description                                       |
|---------------------------|----------------|---------------------------------------------------|
| type                      | string         | constant of string "out_of_gas"                   |

#### move_abort

Object representing the abort condition raised by Move code via `abort` or `assert` during execution of a transaction by the VM on the blockchain.

```
{ type: "move_abort", location: string, abort_code: unsigned int64, explanation: object MoveAbortExplanation or "null" }
```


| Name          | Type                                                        | Description                                                                                                                                    |
| ------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| type          | string                                                      | constant of string "move_abort"                                                                                                                |
| location      | string                                                      | String of the form "address::moduleName" where the abort condition was triggered. "Script" if the abort was raised in the transaction script   |
| abort_code    | unsigned int64                                              | Abort code raised by the Move module                                                                                                           |
| explanation   | [MoveAbortExplanation](#type-moveabortexplanation)>         | Human readable explanation for abort code. "null" if no explanation found.                                                                     |

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

### Type MoveAbortExplanation

a `MoveAbortExplanation` is an object containing globally-defined categories for the abort error e.g., `INVALID_ARGUMENT` along with the Move-module-specific
error reason for the error e.g., `EPAYEE_CANT_ACCEPT_CURRENCY_TYPE`. Both the category and reason are augmented with human-readable descriptions for each.

### Example

```
   {
      "category":"INVALID_ARGUMENT",
      "category_description":" An argument provided to an operation is invalid. Example: a signing key has the wrong format.",
      "reason":"EPAYEE_CANT_ACCEPT_CURRENCY_TYPE",
      "reason_description":" Attempted to send funds in a currency that the receiving account does not hold.\n e.g., `Diem<XDX> to an account that exists, but does not have a `Balance<XDX>` resource"
   }
```

```
{ category: string, category_description: string, reason: string, reason_description: string }
```

| Name                      | Type           | Description                                                           |
|---------------------------|----------------|-----------------------------------------------------------------------|
| category                  | string         | Globally-defined error category                                       |
| category_description      | string         | Description of the error category                                     |
| reason                    | string         | Module-specific error reason                                          |
| reason_description        | string         | Description of the error reason                                       |
