# Overview of Diem Transaction Scripts


> {{move-toc}}

## Introduction

On-chain state is updated via the execution of transaction scripts sent from
accounts that exist on-chain. This page documents each allowed transaction
script on Diem, and the common state changes that can be performed to the
blockchain via these transaction scripts along with their arguments and common
error conditions.

The execution of a transaction script can result in a number of different error
conditions and statuses being returned for each transaction that is committed
on-chain. These statuses and errors can be categorized into two buckets:
* [Predefined statuses](#predefined-statuses): are specific statuses that are returned from the VM, e.g., `OutOfGas`, or `Executed`; and
* [Move Abort errors](#move-aborts): are errors that are raised from the Move modules and/or scripts published on-chain.

There are also a number of statuses that can be returned at the time of
submission of the transaction to the system through JSON-RPC, these are detailed in the
[JSON-RPC specification](https://github.com/diem/diem/blob/main/json-rpc/docs/method_submit.md#errors).

### Predefined Statuses

The predefined set of runtime statuses that can be returned to the user as a
result of executing any transaction script is given by the following table:

| Name                     | Description                                                                                              |
| ----                     | ---                                                                                                      |
| `Executed`               | The transaction was executed successfully.                                                               |
| `OutOfGas`               | The transaction ran out of gas during execution.                                                         |
| `MiscellaneousError`     | The transaction was malformed, e.g., an argument was not in BCS format. Possible, but unlikely to occur. |
| `ExecutionFailure{ ...}` | The transaction encountered an uncaught error. Possible, but unlikely to occur.                          |

**This set of statuses is considered stable**, and they should not be expected to
change. Any changes will be publicized and an upgrade process will be outlined
if/when these statuses or their meanings are updated.

### Move Aborts

Each Move abort error status consists of two pieces of data:
* The Move `location` where the abort was raised. This can be either from within a `Script` or from within a specific `Module`.
* The `abort_code` that was raised.

The `abort_code` is a `u64` that is constructed from two values:
1. The **error category** which is encoded in the lower 8 bits of the code. Error categories are
   declared in the `Errors` module and are globally unique across the Diem framework. There is a limited
   fixed set of predefined categories, and the framework is guaranteed to use these consistently.
2. The **error reason** which is encoded in the remaining 56 bits of the code. The reason is a unique
   number relative to the module which raised the error and can be used to obtain more information about
   the error at hand. It should primarily be used for diagnosis purposes. Error reasons may change over time as the
   framework evolves.

The most common set of Move abort errors that can be returned depend on the transaction script
and they are therefore detailed in the documentation for each transaction
script. Each abort condition is broken down into its category, reason, and a
description of the error in the context of the particular transaction script
e.g.,

| Error Category           | Error Reason                                | Description                                               |
| ----------------         | --------------                              | -------------                                             |
| `Errors::NOT_PUBLISHED`  | `DiemAccount::EPAYER_DOESNT_HOLD_CURRENCY` | `payer` doesn't hold a balance in `Currency`.             |
| `Errors::LIMIT_EXCEEDED` | `DiemAccount::EINSUFFICIENT_BALANCE`       | `amount` is greater than `payer`'s balance in `Currency`. |

For each of these tables, the **error categories should be considered stable**;
any changes to these categories will be be well-publicized in advance. On the
other hand, the **error reasons should be considered only semi-stable**; changes
to these may occur without notice, but changes are not expected to be common.

#### Move Explain

The abort conditions detailed in each transaction script are not meant to
be complete, but the list of error categories are. Additionally, any abort conditions
raised will have a human readable explanation attached to it (if possible) in the
[response](https://github.com/diem/diem/blob/main/json-rpc/docs/type_transaction.md#type-moveabortexplanation)
from a
[JSON-RPC query for a committed transaction](https://github.com/diem/diem/blob/main/json-rpc/json-rpc-spec.md).
These explanations are based off of the human-understandable explanations provided by the
[Move Explain](https://github.com/diem/diem/tree/main/language/tools/move-explain)
tool which can also be called on the command-line.

### Specifications

Transaction scripts come together with formal specifications. See [this document](./spec_documentation.md)
for a discussion of specifications and pointers to further documentation.

---
## Transaction Script Summaries
---

The set of transaction scripts that are allowed to be sent to the blockchain
can be categorized into six different buckets:
* [Account Creation](#account-creation)
* [Account Administration](#account-administration)
* [Payments](#payments)
* [Validator and Validator Operator Administration](#validator-and-validator-operator-administration)
* [Treasury and Compliance Operations](#treasury-and-compliance-operations)
* [System Administration](#system-administration)

This section contains a brief summary for each along with a link to the script's
detailed documentation. The entire list of detailed documentation for each
transaction script categorized in the same manner as here can be found in the
[transaction scripts](#transaction-scripts) section in this document.

### Account Creation

---
#### Script create_child_vasp_account

Creates a Child VASP account with its parent being the sending account of the transaction.
The sender of the transaction must be a Parent VASP account.

Script documentation: `AccountCreationScripts::create_child_vasp_account`

---
#### Script create_validator_operator_account

Creates a Validator Operator account. This transaction can only be sent by the Diem
Root account.

Script documentation: `AccountCreationScripts::create_validator_operator_account`

---
#### Script create_validator_account

Creates a Validator account. This transaction can only be sent by the Diem
Root account.

Script documentation: `AccountCreationScripts::create_validator_account`

---
#### Script create_parent_vasp_account

Creates a Parent VASP account with the specified human name. Must be called by the Treasury Compliance account.

Script documentation: `AccountCreationScripts::create_parent_vasp_account`


---
#### Script create_designated_dealer

Creates a Designated Dealer account with the provided information, and initializes it with
default mint tiers. The transaction can only be sent by the Treasury Compliance account.

Script documentation: `AccountCreationScripts::create_designated_dealer`


### Account Administration

---
#### Script add_currency_to_account

Adds a zero `Currency` balance to the sending `account`. This will enable `account` to
send, receive, and hold `Diem::Diem<Currency>` coins. This transaction can be
successfully sent by any account that is allowed to hold balances
(e.g., VASP, Designated Dealer).

Script documentation: `AccountAdministrationScripts::add_currency_to_account`


---
#### Script add_recovery_rotation_capability

Stores the sending accounts ability to rotate its authentication key with a designated recovery
account. Both the sending and recovery accounts need to belong to the same VASP and
both be VASP accounts. After this transaction both the sending account and the
specified recovery account can rotate the sender account's authentication key.

Script documentation: `AccountAdministrationScripts::add_recovery_rotation_capability`


---
#### Script publish_shared_ed25519_public_key

Rotates the authentication key of the sending account to the
newly-specified public key and publishes a new shared authentication key
under the sender's account. Any account can send this transaction.

Script documentation: `AccountAdministrationScripts::publish_shared_ed25519_public_key`


---
#### Script rotate_authentication_key

Rotates the transaction sender's authentication key to the supplied new authentication key. May
be sent by any account.

Script documentation: `AccountAdministrationScripts::rotate_authentication_key`


---
#### Script rotate_authentication_key_with_nonce

Rotates the sender's authentication key to the supplied new authentication key. May be sent by
any account that has a sliding nonce resource published under it (usually this is Treasury
Compliance or Diem Root accounts).

Script documentation: `AccountAdministrationScripts::rotate_authentication_key_with_nonce`


---
#### Script rotate_authentication_key_with_nonce_admin

Rotates the specified account's authentication key to the supplied new authentication key. May
only be sent by the Diem Root account as a write set transaction.


Script documentation: `AccountAdministrationScripts::rotate_authentication_key_with_nonce_admin`


---
#### Script rotate_authentication_key_with_recovery_address

Rotates the authentication key of a specified account that is part of a recovery address to a
new authentication key. Only used for accounts that are part of a recovery address (see
`AccountAdministrationScripts::add_recovery_rotation_capability` for account restrictions).

Script documentation: `AccountAdministrationScripts::rotate_authentication_key_with_recovery_address`


---
#### Script rotate_dual_attestation_info

Updates the url used for off-chain communication, and the public key used to verify dual
attestation on-chain. Transaction can be sent by any account that has dual attestation
information published under it. In practice the only such accounts are Designated Dealers and
Parent VASPs.

Script documentation: `AccountAdministrationScripts::rotate_dual_attestation_info`


---
#### Script rotate_shared_ed25519_public_key

Rotates the authentication key in a `SharedEd25519PublicKey`. This transaction can be sent by
any account that has previously published a shared ed25519 public key using
`AccountAdministrationScripts::publish_shared_ed25519_public_key`.

Script documentation: `AccountAdministrationScripts::rotate_shared_ed25519_public_key`

### Payments

---
#### Script peer_to_peer_with_metadata

Transfers a given number of coins in a specified currency from one account to another.
Transfers over a specified amount defined on-chain that are between two different VASPs, or
other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
agreed to receive the coins.  This transaction can be sent by any account that can hold a
balance, and to any account that can hold a balance. Both accounts must hold balances in the
currency being transacted.

Script documentation: `PaymentScripts::peer_to_peer_with_metadata`


### Validator and Validator Operator Administration

---
#### Script add_validator_and_reconfigure

Adds a validator account to the validator set, and triggers a
reconfiguration of the system to admit the account to the validator set for the system. This
transaction can only be successfully called by the Diem Root account.

Script documentation: `ValidatorAdministrationScripts::add_validator_and_reconfigure`


---
#### Script register_validator_config

Updates a validator's configuration. This does not reconfigure the system and will not update
the configuration in the validator set that is seen by other validators in the network. Can
only be successfully sent by a Validator Operator account that is already registered with a
validator.

Script documentation: `ValidatorAdministrationScripts::register_validator_config`


---
#### Script remove_validator_and_reconfigure

This script removes a validator account from the validator set, and triggers a reconfiguration
of the system to remove the validator from the system. This transaction can only be
successfully called by the Diem Root account.

Script documentation: `ValidatorAdministrationScripts::remove_validator_and_reconfigure`


---
#### Script set_validator_config_and_reconfigure

Updates a validator's configuration, and triggers a reconfiguration of the system to update the
validator set with this new validator configuration.  Can only be successfully sent by a
Validator Operator account that is already registered with a validator.

Script documentation: `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`


---
#### Script set_validator_operator

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by an account with
Validator role.

Script documentation: `ValidatorAdministrationScripts::set_validator_operator`


---
#### Script set_validator_operator_with_nonce_admin

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by the Diem Root
account as a write set transaction.

Script documentation: `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`


### Treasury and Compliance Operations

---
#### Script preburn

Moves a specified number of coins in a given currency from the account's
balance to its preburn area after which the coins may be burned. This
transaction may be sent by any account that holds a balance and preburn area
in the specified currency.

Script documentation: `TreasuryComplianceScripts::preburn`


---
#### Script burn_with_amount

Burns the coins held in a preburn resource in the preburn queue at the
specified preburn address, which are equal to the `amount` specified in the
transaction. Finds the first relevant outstanding preburn request with
matching amount and removes the contained coins from the system. The sending
account must be the Treasury Compliance account.
The account that holds the preburn queue resource will normally be a Designated
Dealer, but there are no enforced requirements that it be one.

Script documentation: `TreasuryComplianceScripts::burn`


---
#### Script cancel_burn_with_amount

Cancels and returns the coins held in the preburn area under
`preburn_address`, which are equal to the `amount` specified in the transaction. Finds the first preburn
resource with the matching amount and returns the funds to the `preburn_address`'s balance.
Can only be successfully sent by an account with Treasury Compliance role.

Script documentation: `TreasuryComplianceScripts::cancel_burn`


---
#### Script burn_txn_fees

Burns the transaction fees collected in the `CoinType` currency so that the
Diem association may reclaim the backing coins off-chain. May only be sent
by the Treasury Compliance account.

Script documentation: `TreasuryComplianceScripts::burn_txn_fees`


---
#### Script tiered_mint

Mints a specified number of coins in a currency to a Designated Dealer. The sending account
must be the Treasury Compliance account, and coins can only be minted to a Designated Dealer
account.

Script documentation: `TreasuryComplianceScripts::tiered_mint`


---
#### Script freeze_account

Freezes the account at `address`. The sending account of this transaction
must be the Treasury Compliance account. The account being frozen cannot be
the Diem Root or Treasury Compliance account. After the successful
execution of this transaction no transactions may be sent from the frozen
account, and the frozen account may not send or receive coins.

Script documentation: `TreasuryComplianceScripts::freeze_account`


---
#### Script unfreeze_account

Unfreezes the account at `address`. The sending account of this transaction must be the
Treasury Compliance account. After the successful execution of this transaction transactions
may be sent from the previously frozen account, and coins may be sent and received.

Script documentation: `TreasuryComplianceScripts::unfreeze_account`


---
#### Script update_dual_attestation_limit

Update the dual attestation limit on-chain. Defined in terms of micro-XDX.  The transaction can
only be sent by the Treasury Compliance account.  After this transaction all inter-VASP
payments over this limit must be checked for dual attestation.

Script documentation: `TreasuryComplianceScripts::update_dual_attestation_limit`


---
#### Script update_exchange_rate

Update the rough on-chain exchange rate between a specified currency and XDX (as a conversion
to micro-XDX). The transaction can only be sent by the Treasury Compliance account. After this
transaction the updated exchange rate will be used for normalization of gas prices, and for
dual attestation checking.

Script documentation: `TreasuryComplianceScripts::update_exchange_rate`


---
#### Script update_minting_ability

Script to allow or disallow minting of new coins in a specified currency.  This transaction can
only be sent by the Treasury Compliance account.  Turning minting off for a currency will have
no effect on coins already in circulation, and coins may still be removed from the system.

Script documentation: `TreasuryComplianceScripts::update_minting_ability`


### System Administration

---
#### Script update_diem_version

Updates the Diem major version that is stored on-chain and is used by the VM.  This
transaction can only be sent from the Diem Root account.

Script documentation: `SystemAdministrationScripts::update_diem_version`



---
## Transaction Scripts
---

### Account Creation

> {{move-include AccountCreationScripts}}

---
### Account Administration

> {{move-include AccountAdministrationScripts}}

---
### Payments

> {{move-include PaymentScripts}}

---
### Validator and Validator Operator Administration

> {{move-include ValidatorAdministrationScripts}}

---
### Treasury and Compliance Operations

> {{move-include TreasuryComplianceScripts}}

---
### System Administration

> {{move-include SystemAdministrationScripts}}

### Index

> {{move-index}}
