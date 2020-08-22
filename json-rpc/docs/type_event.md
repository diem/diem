## Type Event

**Description**

An event emitted during a transaction


### Attributes

| Name                | Type                     | Description                                                    |
|---------------------|--------------------------|----------------------------------------------------------------|
| key                 | string                   | Gobally unique identifier of event stream                      |
| sequence_number     | string                   | Sequence number of the current event in the given even stream  |
| transaction_version | unsigned int64           | Version of the transaction that emitted this event             |
| data                | [EventData](#event-data) | Typed event data object                                        |


### Event Data

Event data is serialized into one JSON object with a "type" field to indicate it's type.

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | Type name of EventData                                         |

#### burn

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | constant string "burn"                                         |
| amount              | [Amount](type_amount.md) | amount burned                                        |
| preburn_address     | string         | preburn account address                                        |

#### cancelburn

| Name                | Type           | Description                                                    |
|---------------------|----------------|----------------------------------------------------------------|
| type                | string         | constant string "cancelburn"                                   |
| amount              | [Amount](type_amount.md) | amount canceled                                      |
| preburn_address     | string         | preburn account address                                        |

#### preburn

| Name                | Type                     | Description                                          |
|---------------------|--------------------------|------------------------------------------------------|
| type                | string                   | constant string "preburn"                            |
| amount              | [Amount](type_amount.md) | amount preburn                                       |
| preburn_address     | string                   | preburn account address                              |

#### mint

| Name                | Type                     | Description                       |
|---------------------|--------------------------|-----------------------------------|
| type                | string                   | constant string "mint"            |
| amount              | [Amount](type_amount.md) | amount mint                       |

#### to_lbr_exchange_rate_update

| Name                     | Type     | Description                                  |
|--------------------------|----------|----------------------------------------------|
| type                     | string   | constant string "to_lbr_exchange_rate_update"|
| currency_code            | string   | currency code of the exchange rate updated   |
| new_to_lbr_exchange_rate | float32  | currency code of the exchange rate updated   |

#### receivedpayment

| Name                | Type                     | Description                           |
|---------------------|--------------------------|---------------------------------------|
| type                | string                   | constant string "receivedpayment"     |
| amount              | [Amount](type_amount.md) | Amount received from the sender of the transaction |
| sender              | string                   | Hex-encoded address of the account whose balance was debited to perform this deposit. If the deposited funds came from a mint, the sender address will be 0x0...0. |
| receiver            | string                   | Hex-encoded address of the account whose balance was credited by this deposit. |
| metadata            | string                   | An optional field that can contain extra metadata for the event (from RawTransaction metadata??). This information can be used by an off-chain API to implement a sub-addressing scheme for a wallet. |

#### sentpayment

Event emitted when an account sends a payment.

| Name                | Type                     | Description                           |
|---------------------|--------------------------|---------------------------------------|
| type                | string                   | constant string "sentpayment"         |
| amount              | [Amount](type_amount.md) | Amount transferred in a transaction   |
| sender              | string                   | Hex-encoded address of the account whose balance was debited to perform this deposit. If the deposited funds came from a mint, the sender address will be 0x0...0. |
| receiver            | string                   | Hex-encoded address of the account whose balance was credited by this deposit. |
| metadata            | string                   | An optional field that can contain extra metadata for the event (from RawTransaction metadata??). This information can be used by an off-chain API to implement a sub-addressing scheme for a wallet. |

#### compliancekeyrotation

Event emitted when the public key used for dual attestation checking on-chain is rotated. Event key can be found in the `compliance_key_rotation_event_key` field for parent VASPs and designated dealers.

| Name                        | Type   | Description                                             |
|-----------------------------|--------|---------------------------------------------------------|
| type                        | string | constant string "compliancekeyrotation"                 |
| new_compliance_public_key   | string | Hex-encoded new dual attestation compliance public key  |
| time_rotated_seconds        | u64    | Blockchain time (in seconds) when the rotation occurred |

#### baseurlrotation

Event emitted when the url used for off-chain dual attestation checking is rotated on-chain.  Event key can be found in the `base_url_rotation_event_key` field for parent VASPs and designated dealers.

| Name                        | Type   | Description                                             |
|-----------------------------|--------|---------------------------------------------------------|
| type                        | string | constant string "baseurlrotation"                       |
| new_base_url                | string | New URL endpoint for off-chain communication            |
| time_rotated_seconds        | u64    | Blockchain time (in seconds) when the rotation occurred |

#### unknown

Represents events currently unsupported by JSON-RPC API.

| Name    | Type   | Description                 |
|---------|--------|-----------------------------|
| type    | string | Constant string "unknown"   |

[1]: https://libra.github.io/libra/libra_canonical_serialization/index.html "LCS"
