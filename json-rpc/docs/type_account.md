## Type Account

**Description**

A Diem account.


### Attributes

| Name                              | Type                           | Description                                                                                 |
|-----------------------------------|--------------------------------|---------------------------------------------------------------------------------------------|
| address                           | string                         | the account address                                                                         |
| sequence_number                   | unsigned int64                 | The next sequence number for the current account                                            |
| authentication_key                | string                         | Hex-encoded authentication key for the account                                              |
| delegated_key_rotation_capability | boolean                        | If true, another account has the ability to rotate the authentication key for this account. |
| delegated_withdrawal_capability   | boolean                        | If true, another account has the ability to withdraw funds from this account.               |
| balances                          | List<[Amount](type_amount.md)> | Balances of all the currencies associated with the account                                  |
| sent_events_key                   | string                         | Unique key for the sent events stream of this account                                       |
| received_events_key               | string                         | Unique key for the received events stream of this account                                   |
| is_frozen                         | boolean                        | Whether this account is frozen or not                                                       |
| role                              | object                         | One of the following type:                                                                |
|                                   |                                |   - [UnknownRole](#type-unknownrole) |
|                                   |                                |   - [ParentVASPRole](#type-parentvasprole) |
|                                   |                                |   - [ChildVASPRole](#type-childvasprole) |
|                                   |                                |   - [DesignatedDealerRole](#type-designateddealerrole) |

---

## Type DesignatedDealerRole

### Attributes

| Name                               | Type                           | Description                                                       |
|------------------------------------|--------------------------------|-------------------------------------------------------------------|
| type                               | string                         | "designated_dealer"                                               |
| human_name                         | string                         | human-readable name of this designated dealer                     |
| base_url                           | string                         | base URL for this designated dealer                               |
| expiration_time                    | unsigned int64(microseconds)   | expiration time for this designated dealer                        |
| compliance_key                     | string                         | compliance key for this designated dealer                         |
| preburn balances                   | List<[Amount](type_amount.md)> | Preburn balances of this designated dealer                        |
| received_mint_events_key           | string                         | key of received mint events for this designated dealer            |
| compliance_key_rotation_events_key | string                         | key of compliance key rotation events for this designated dealer  |
| base_url_rotation_events_key       | string                         | key of base url key rotation events for this designated dealer    |


---

## Type ParentVASPRole

### Attributes

| Name                               | Type                         | Description                                                       |
|------------------------------------|------------------------------|-------------------------------------------------------------------|
| type                               | string                       | "parent_vasp"                                                     |
| human_name                         | string                       | human-readable name of this parent VASP                           |
| base_url                           | string                       | base URL for this parent VASP                                     |
| expiration_time                    | unsigned int64(microseconds) | expiration time for this parent VASP                              |
| compliance_key                     | string                       | compliance key for this parent VASP                               |
| num_children                       | unsigned int64               | number of children of this parent VASP                            |
| compliance_key_rotation_events_key | string                       | key of compliance key rotation events for this parent VASP        |
| base_url_rotation_events_key       | string                       | key of base url key rotation events for this parent VASP          |



---



## Type ChildVASPRole

### Attributes

| Name                | Type   | Description                                |
|---------------------|--------|--------------------------------------------|
| type                | string | "child_vasp"                               |
| parent_vasp_address | string | address of this child VASP's parent VASP   |


---

## Type UnknownRole

### Attributes

| Name                | Type   | Description                                |
|---------------------|--------|--------------------------------------------|
| type                | string | "unknown"                                  |
