
<a name="@Overview_of_Libra_Transaction_Scripts_0"></a>

# Overview of Libra Transaction Scripts


-  [Background](#@Background_1)
    -  [Predefined Statuses](#@Predefined_Statuses_2)
    -  [Move Aborts](#@Move_Aborts_3)
        -  [Move Explain](#@Move_Explain_4)
-  [Transaction Script Summaries](#@Transaction_Script_Summaries_5)
    -  [Account Creation](#@Account_Creation_6)
        -  [Script create_child_vasp_account](#@Script_create_child_vasp_account_7)
        -  [Script create_validator_operator_account](#@Script_create_validator_operator_account_8)
        -  [Script create_validator_account](#@Script_create_validator_account_9)
        -  [Script create_parent_vasp_account](#@Script_create_parent_vasp_account_10)
        -  [Script create_designated_dealer](#@Script_create_designated_dealer_11)
    -  [Account Administration](#@Account_Administration_12)
        -  [Script add_currency_to_account](#@Script_add_currency_to_account_13)
        -  [Script add_recovery_rotation_capability](#@Script_add_recovery_rotation_capability_14)
        -  [Script publish_shared_ed25519_public_key](#@Script_publish_shared_ed25519_public_key_15)
        -  [Script rotate_authentication_key](#@Script_rotate_authentication_key_16)
        -  [Script rotate_authentication_key_with_nonce](#@Script_rotate_authentication_key_with_nonce_17)
        -  [Script rotate_authentication_key_with_nonce_admin](#@Script_rotate_authentication_key_with_nonce_admin_18)
        -  [Script rotate_authentication_key_with_recovery_address](#@Script_rotate_authentication_key_with_recovery_address_19)
        -  [Script rotate_dual_attestation_info](#@Script_rotate_dual_attestation_info_20)
        -  [Script rotate_shared_ed25519_public_key](#@Script_rotate_shared_ed25519_public_key_21)
        -  [Script mint_lbr](#@Script_mint_lbr_22)
        -  [Script unmint_lbr](#@Script_unmint_lbr_23)
    -  [Payments](#@Payments_24)
        -  [Script peer_to_peer_with_metadata](#@Script_peer_to_peer_with_metadata_25)
    -  [Validator and Validator Operator Administration](#@Validator_and_Validator_Operator_Administration_26)
        -  [Script add_validator_and_reconfigure](#@Script_add_validator_and_reconfigure_27)
        -  [Script register_validator_config](#@Script_register_validator_config_28)
        -  [Script remove_validator_and_reconfigure](#@Script_remove_validator_and_reconfigure_29)
        -  [Script set_validator_config_and_reconfigure](#@Script_set_validator_config_and_reconfigure_30)
        -  [Script set_validator_operator](#@Script_set_validator_operator_31)
        -  [Script set_validator_operator_with_nonce_admin](#@Script_set_validator_operator_with_nonce_admin_32)
    -  [Treasury and Compliance Operations](#@Treasury_and_Compliance_Operations_33)
        -  [Script preburn](#@Script_preburn_34)
        -  [Script burn](#@Script_burn_35)
        -  [Script cancel_burn](#@Script_cancel_burn_36)
        -  [Script burn_txn_fees](#@Script_burn_txn_fees_37)
        -  [Script tiered_mint](#@Script_tiered_mint_38)
        -  [Script freeze_account](#@Script_freeze_account_39)
        -  [Script unfreeze_account](#@Script_unfreeze_account_40)
        -  [Script update_dual_attestation_limit](#@Script_update_dual_attestation_limit_41)
        -  [Script update_exchange_rate](#@Script_update_exchange_rate_42)
        -  [Script update_minting_ability](#@Script_update_minting_ability_43)
    -  [System Administration](#@System_Administration_44)
        -  [Script update_libra_version](#@Script_update_libra_version_45)
        -  [Script add_to_script_allow_list](#@Script_add_to_script_allow_list_46)
-  [Transaction Scripts](#@Transaction_Scripts_47)
    -  [Account Creation](#@Account_Creation_48)
        -  [Script <code><a href="overview.md#create_child_vasp_account">create_child_vasp_account</a></code>](#create_child_vasp_account)
        -  [Script <code><a href="overview.md#create_validator_operator_account">create_validator_operator_account</a></code>](#create_validator_operator_account)
        -  [Script <code><a href="overview.md#create_validator_account">create_validator_account</a></code>](#create_validator_account)
        -  [Script <code><a href="overview.md#create_parent_vasp_account">create_parent_vasp_account</a></code>](#create_parent_vasp_account)
        -  [Script <code><a href="overview.md#create_designated_dealer">create_designated_dealer</a></code>](#create_designated_dealer)
    -  [Account Administration](#@Account_Administration_75)
        -  [Script <code><a href="overview.md#add_currency_to_account">add_currency_to_account</a></code>](#add_currency_to_account)
        -  [Script <code><a href="overview.md#add_recovery_rotation_capability">add_recovery_rotation_capability</a></code>](#add_recovery_rotation_capability)
        -  [Script <code><a href="overview.md#publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a></code>](#publish_shared_ed25519_public_key)
        -  [Script <code><a href="overview.md#create_recovery_address">create_recovery_address</a></code>](#create_recovery_address)
        -  [Script <code><a href="overview.md#rotate_authentication_key">rotate_authentication_key</a></code>](#rotate_authentication_key)
        -  [Script <code><a href="overview.md#rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a></code>](#rotate_authentication_key_with_nonce)
        -  [Script <code><a href="overview.md#rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a></code>](#rotate_authentication_key_with_nonce_admin)
        -  [Script <code><a href="overview.md#rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a></code>](#rotate_authentication_key_with_recovery_address)
        -  [Script <code><a href="overview.md#rotate_dual_attestation_info">rotate_dual_attestation_info</a></code>](#rotate_dual_attestation_info)
        -  [Script <code><a href="overview.md#rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a></code>](#rotate_shared_ed25519_public_key)
        -  [Script <code><a href="overview.md#mint_lbr">mint_lbr</a></code>](#mint_lbr)
        -  [Script <code><a href="overview.md#unmint_lbr">unmint_lbr</a></code>](#unmint_lbr)
    -  [Payments](#@Payments_139)
        -  [Script <code><a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a></code>](#peer_to_peer_with_metadata)
    -  [Validator and Validator Operator Administration](#@Validator_and_Validator_Operator_Administration_146)
        -  [Script <code><a href="overview.md#add_validator_and_reconfigure">add_validator_and_reconfigure</a></code>](#add_validator_and_reconfigure)
        -  [Script <code><a href="overview.md#register_validator_config">register_validator_config</a></code>](#register_validator_config)
        -  [Script <code><a href="overview.md#remove_validator_and_reconfigure">remove_validator_and_reconfigure</a></code>](#remove_validator_and_reconfigure)
        -  [Script <code><a href="overview.md#set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a></code>](#set_validator_config_and_reconfigure)
        -  [Script <code><a href="overview.md#set_validator_operator">set_validator_operator</a></code>](#set_validator_operator)
        -  [Script <code><a href="overview.md#set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a></code>](#set_validator_operator_with_nonce_admin)
    -  [Treasury and Compliance Operations](#@Treasury_and_Compliance_Operations_177)
        -  [Script <code><a href="overview.md#preburn">preburn</a></code>](#preburn)
        -  [Script <code><a href="overview.md#burn">burn</a></code>](#burn)
        -  [Script <code><a href="overview.md#cancel_burn">cancel_burn</a></code>](#cancel_burn)
        -  [Script <code><a href="overview.md#burn_txn_fees">burn_txn_fees</a></code>](#burn_txn_fees)
        -  [Script <code><a href="overview.md#tiered_mint">tiered_mint</a></code>](#tiered_mint)
        -  [Script <code><a href="overview.md#freeze_account">freeze_account</a></code>](#freeze_account)
        -  [Script <code><a href="overview.md#unfreeze_account">unfreeze_account</a></code>](#unfreeze_account)
        -  [Script <code><a href="overview.md#update_dual_attestation_limit">update_dual_attestation_limit</a></code>](#update_dual_attestation_limit)
        -  [Script <code><a href="overview.md#update_exchange_rate">update_exchange_rate</a></code>](#update_exchange_rate)
        -  [Script <code><a href="overview.md#update_minting_ability">update_minting_ability</a></code>](#update_minting_ability)
    -  [System Administration](#@System_Administration_235)
        -  [Script <code><a href="overview.md#update_libra_version">update_libra_version</a></code>](#update_libra_version)
        -  [Script <code><a href="overview.md#add_to_script_allow_list">add_to_script_allow_list</a></code>](#add_to_script_allow_list)
    -  [Index](#@Index_244)



<a name="@Background_1"></a>

## Background

Executing a transaction script can result in a number of different error
conditions and statuses being returned for a transaction that is committed
on-chain. These can be categorized into two buckets:
* [Predefined statuses](#predefined-statuses): are specific statuses that are returned from the VM, e.g., <code>OutOfGas</code>, or <code>Executed</code>; and
* [Move Abort errors](#move-aborts): are errors that are raised from the Move modules and/or scripts published on-chain.

There are also a number of statuses that can be returned at the time of
submission of the transaction to the system through JSON-RPC, these are detailed in the
[JSON-RPC specification](https://github.com/libra/libra/blob/master/json-rpc/docs/method_submit.md#errors).


<a name="@Predefined_Statuses_2"></a>

### Predefined Statuses


The predefined set of runtime statuses that can be returned to the user as a
result of executing any transaction script is given by the following table:

| Name                     | Description                                                                                              |
| ----                     | ---                                                                                                      |
| <code>Executed</code>               | The transaction was executed successfully.                                                               |
| <code>OutOfGas</code>               | The transaction ran out of gas during execution.                                                         |
| <code>MiscellaneousError</code>     | The transaction was malformed, e.g., an argument was not in LCS format. Possible, but unlikely to occur. |
| <code>ExecutionFailure{ ...}</code> | The transaction encountered an uncaught error. Possible, but unlikely to occur.                          |

**This set of statuses is considered stable**, and they should not be expected to
change. Any changes will be publicized and an upgrade process will be outlined
if/when these statuses or their meanings are updated.


<a name="@Move_Aborts_3"></a>

### Move Aborts


Each Move abort error status consists of two pieces of data:
* The Move <code>location</code> where the abort was raised. This can be either from within a <code>Script</code> or from within a specific <code>Module</code>.
* The <code>abort_code</code> that was raised.

The <code>abort_code</code> is a <code>u64</code> that is constructed from two values:
1. The **error category** which is encoded in the lower 8 bits of the code. Error categories are
declared in the <code><a href="../../modules/doc/Errors.md#0x1_Errors">Errors</a></code> module and are globally unique across the Libra framework. There is a limited
fixed set of predefined categories, and the framework is guaranteed to use these consistently.
2. The **error reason** which is encoded in the remaining 54 bits of the code. The reason is a unique
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
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code> | <code>payer</code> doesn't hold a balance in <code>Currency</code>.             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>       | <code>amount</code> is greater than <code>payer</code>'s balance in <code>Currency</code>. |

For each of these tables, the **error categories should be considered stable**;
any changes to these categories will be be well-publicized in advance. On the
other hand, the **error reasons should be considered only semi-stable**; changes
to these may occur without notice, but changes are not expected to be common.


<a name="@Move_Explain_4"></a>

#### Move Explain


The abort conditions detailed in each transaction script are not meant to
be complete, but the list of error categories are. Additionally, any abort conditions
raised will have a human readable explanation attached to it (if possible) in the
[response](https://github.com/libra/libra/blob/master/json-rpc/docs/type_transaction.md#type-moveabortexplanation)
from a
[JSON-RPC query for a committed transaction](https://github.com/libra/libra/blob/master/json-rpc/json-rpc-spec.md).
These explanations are based off of the human-understandable explanations provided by the
[Move Explain](https://github.com/libra/libra/tree/master/language/tools/move-explain)
tool which can also be called on the command-line.

---

<a name="@Transaction_Script_Summaries_5"></a>

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


<a name="@Account_Creation_6"></a>

### Account Creation


---

<a name="@Script_create_child_vasp_account_7"></a>

#### Script create_child_vasp_account


Creates a Child VASP account with its parent being the sending account of the transaction.
The sender of the transaction must be a Parent VASP account.

Script documentation: <code><a href="overview.md#create_child_vasp_account">create_child_vasp_account</a></code>

---

<a name="@Script_create_validator_operator_account_8"></a>

#### Script create_validator_operator_account


Creates a Validator Operator account. This transaction can only be sent by the Libra
Root account.

Script documentation: <code><a href="overview.md#create_validator_operator_account">create_validator_operator_account</a></code>

---

<a name="@Script_create_validator_account_9"></a>

#### Script create_validator_account


Creates a Validator account. This transaction can only be sent by the Libra
Root account.

Script documentation: <code><a href="overview.md#create_validator_account">create_validator_account</a></code>

---

<a name="@Script_create_parent_vasp_account_10"></a>

#### Script create_parent_vasp_account


Creates a Parent VASP account with the specified human name. Must be called by the Treasury Compliance account.

Script documentation: <code><a href="overview.md#create_parent_vasp_account">create_parent_vasp_account</a></code>


---

<a name="@Script_create_designated_dealer_11"></a>

#### Script create_designated_dealer


Creates a Designated Dealer account with the provided information, and initializes it with
default mint tiers. The transaction can only be sent by the Treasury Compliance account.

Script documentation: <code><a href="overview.md#create_designated_dealer">create_designated_dealer</a></code>



<a name="@Account_Administration_12"></a>

### Account Administration


---

<a name="@Script_add_currency_to_account_13"></a>

#### Script add_currency_to_account


Adds a zero <code>Currency</code> balance to the sending <code>account</code>. This will enable <code>account</code> to
send, receive, and hold <code><a href="../../modules/doc/Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Currency&gt;</code> coins. This transaction can be
successfully sent by any account that is allowed to hold balances
(e.g., VASP, Designated Dealer).

Script documentation: <code><a href="overview.md#add_currency_to_account">add_currency_to_account</a></code>


---

<a name="@Script_add_recovery_rotation_capability_14"></a>

#### Script add_recovery_rotation_capability


Stores the sending accounts ability to rotate its authentication key with a designated recovery
account. Both the sending and recovery accounts need to belong to the same VASP and
both be VASP accounts. After this transaction both the sending account and the
specified recovery account can rotate the sender account's authentication key.

Script documentation: <code><a href="overview.md#add_recovery_rotation_capability">add_recovery_rotation_capability</a></code>


---

<a name="@Script_publish_shared_ed25519_public_key_15"></a>

#### Script publish_shared_ed25519_public_key


Rotates the authentication key of the sending account to the
newly-specified public key and publishes a new shared authentication key
under the sender's account. Any account can send this transaction.

Script documentation: <code><a href="overview.md#publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a></code>


---

<a name="@Script_rotate_authentication_key_16"></a>

#### Script rotate_authentication_key


Rotates the transaction sender's authentication key to the supplied new authentication key. May
be sent by any account.

Script documentation: <code><a href="overview.md#rotate_authentication_key">rotate_authentication_key</a></code>


---

<a name="@Script_rotate_authentication_key_with_nonce_17"></a>

#### Script rotate_authentication_key_with_nonce


Rotates the sender's authentication key to the supplied new authentication key. May be sent by
any account that has a sliding nonce resource published under it (usually this is Treasury
Compliance or Libra Root accounts).

Script documentation: <code><a href="overview.md#rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a></code>


---

<a name="@Script_rotate_authentication_key_with_nonce_admin_18"></a>

#### Script rotate_authentication_key_with_nonce_admin


Rotates the specified account's authentication key to the supplied new authentication key. May
only be sent by the Libra Root account as a write set transaction.


Script documentation: <code><a href="overview.md#rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a></code>


---

<a name="@Script_rotate_authentication_key_with_recovery_address_19"></a>

#### Script rotate_authentication_key_with_recovery_address


Rotates the authentication key of a specified account that is part of a recovery address to a
new authentication key. Only used for accounts that are part of a recovery address (see
<code><a href="overview.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code> for account restrictions).

Script documentation: <code><a href="overview.md#rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a></code>


---

<a name="@Script_rotate_dual_attestation_info_20"></a>

#### Script rotate_dual_attestation_info


Updates the url used for off-chain communication, and the public key used to verify dual
attestation on-chain. Transaction can be sent by any account that has dual attestation
information published under it. In practice the only such accounts are Designated Dealers and
Parent VASPs.

Script documentation: <code><a href="overview.md#rotate_dual_attestation_info">rotate_dual_attestation_info</a></code>


---

<a name="@Script_rotate_shared_ed25519_public_key_21"></a>

#### Script rotate_shared_ed25519_public_key


Rotates the authentication key in a <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code>. This transaction can be sent by
any account that has previously published a shared ed25519 public key using
<code><a href="overview.md#publish_shared_ed25519_public_key">Script::publish_shared_ed25519_public_key</a></code>.

Script documentation: <code><a href="overview.md#rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a></code>


---

<a name="@Script_mint_lbr_22"></a>

#### Script mint_lbr


Mints LBR from the sending account's constituent coins by depositing in the
on-chain LBR reserve. Deposits the newly-minted LBR into the sending
account. Can be sent by any account that can hold balances for the constituent
currencies for LBR and LBR.

Script documentation: <code><a href="overview.md#mint_lbr">mint_lbr</a></code>


---

<a name="@Script_unmint_lbr_23"></a>

#### Script unmint_lbr


Withdraws a specified amount of LBR from the transaction sender's account, and unstaples the
withdrawn LBR into its constituent coins. Deposits each of the constituent coins to the
transaction sender's balances. Any account that can hold balances that has the correct balances
may send this transaction.

Script documentation: <code><a href="overview.md#unmint_lbr">unmint_lbr</a></code>



<a name="@Payments_24"></a>

### Payments


---

<a name="@Script_peer_to_peer_with_metadata_25"></a>

#### Script peer_to_peer_with_metadata


Transfers a given number of coins in a specified currency from one account to another.
Transfers over a specified amount defined on-chain that are between two different VASPs, or
other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
agreed to receive the coins.  This transaction can be sent by any account that can hold a
balance, and to any account that can hold a balance. Both accounts must hold balances in the
currency being transacted.

Script documentation: <code><a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a></code>



<a name="@Validator_and_Validator_Operator_Administration_26"></a>

### Validator and Validator Operator Administration


---

<a name="@Script_add_validator_and_reconfigure_27"></a>

#### Script add_validator_and_reconfigure


Adds a validator account to the validator set, and triggers a
reconfiguration of the system to admit the account to the validator set for the system. This
transaction can only be successfully called by the Libra Root account.

Script documentation: <code><a href="overview.md#add_validator_and_reconfigure">add_validator_and_reconfigure</a></code>


---

<a name="@Script_register_validator_config_28"></a>

#### Script register_validator_config


Updates a validator's configuration. This does not reconfigure the system and will not update
the configuration in the validator set that is seen by other validators in the network. Can
only be successfully sent by a Validator Operator account that is already registered with a
validator.

Script documentation: <code><a href="overview.md#register_validator_config">register_validator_config</a></code>


---

<a name="@Script_remove_validator_and_reconfigure_29"></a>

#### Script remove_validator_and_reconfigure


This script removes a validator account from the validator set, and triggers a reconfiguration
of the system to remove the validator from the system. This transaction can only be
successfully called by the Libra Root account.

Script documentation: <code><a href="overview.md#remove_validator_and_reconfigure">remove_validator_and_reconfigure</a></code>


---

<a name="@Script_set_validator_config_and_reconfigure_30"></a>

#### Script set_validator_config_and_reconfigure


Updates a validator's configuration, and triggers a reconfiguration of the system to update the
validator set with this new validator configuration.  Can only be successfully sent by a
Validator Operator account that is already registered with a validator.

Script documentation: <code><a href="overview.md#set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a></code>


---

<a name="@Script_set_validator_operator_31"></a>

#### Script set_validator_operator


Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by an account with
Validator role.

Script documentation: <code><a href="overview.md#set_validator_operator">set_validator_operator</a></code>


---

<a name="@Script_set_validator_operator_with_nonce_admin_32"></a>

#### Script set_validator_operator_with_nonce_admin


Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by the Libra Root
account as a write set transaction.

Script documentation: <code><a href="overview.md#set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a></code>



<a name="@Treasury_and_Compliance_Operations_33"></a>

### Treasury and Compliance Operations


---

<a name="@Script_preburn_34"></a>

#### Script preburn


Moves a specified number of coins in a given currency from the account's
balance to its preburn area after which the coins may be burned. This
transaction may be sent by any account that holds a balance and preburn area
in the specified currency.

Script documentation: <code><a href="overview.md#preburn">preburn</a></code>


---

<a name="@Script_burn_35"></a>

#### Script burn


Burns all coins held in the preburn resource at the specified
preburn address and removes them from the system. The sending account must
be the Treasury Compliance account.
The account that holds the preburn resource will normally be a Designated
Dealer, but there are no enforced requirements that it be one.

Script documentation: <code><a href="overview.md#burn">burn</a></code>


---

<a name="@Script_cancel_burn_36"></a>

#### Script cancel_burn


Cancels and returns all coins held in the preburn area under
<code>preburn_address</code> and returns the funds to the <code>preburn_address</code>'s balance.
Can only be successfully sent by an account with Treasury Compliance role.

Script documentation: <code><a href="overview.md#cancel_burn">cancel_burn</a></code>


---

<a name="@Script_burn_txn_fees_37"></a>

#### Script burn_txn_fees


Burns the transaction fees collected in the <code>CoinType</code> currency so that the
Libra association may reclaim the backing coins off-chain. May only be sent
by the Treasury Compliance account.

Script documentation: <code><a href="overview.md#burn_txn_fees">burn_txn_fees</a></code>


---

<a name="@Script_tiered_mint_38"></a>

#### Script tiered_mint


Mints a specified number of coins in a currency to a Designated Dealer. The sending account
must be the Treasury Compliance account, and coins can only be minted to a Designated Dealer
account.

Script documentation: <code><a href="overview.md#tiered_mint">tiered_mint</a></code>


---

<a name="@Script_freeze_account_39"></a>

#### Script freeze_account


Freezes the account at <code>address</code>. The sending account of this transaction
must be the Treasury Compliance account. The account being frozen cannot be
the Libra Root or Treasury Compliance account. After the successful
execution of this transaction no transactions may be sent from the frozen
account, and the frozen account may not send or receive coins.

Script documentation: <code><a href="overview.md#freeze_account">freeze_account</a></code>


---

<a name="@Script_unfreeze_account_40"></a>

#### Script unfreeze_account


Unfreezes the account at <code>address</code>. The sending account of this transaction must be the
Treasury Compliance account. After the successful execution of this transaction transactions
may be sent from the previously frozen account, and coins may be sent and received.

Script documentation: <code><a href="overview.md#unfreeze_account">unfreeze_account</a></code>


---

<a name="@Script_update_dual_attestation_limit_41"></a>

#### Script update_dual_attestation_limit


Update the dual attestation limit on-chain. Defined in terms of micro-LBR.  The transaction can
only be sent by the Treasury Compliance account.  After this transaction all inter-VASP
payments over this limit must be checked for dual attestation.

Script documentation: <code><a href="overview.md#update_dual_attestation_limit">update_dual_attestation_limit</a></code>


---

<a name="@Script_update_exchange_rate_42"></a>

#### Script update_exchange_rate


Update the rough on-chain exchange rate between a specified currency and LBR (as a conversion
to micro-LBR). The transaction can only be sent by the Treasury Compliance account. After this
transaction the updated exchange rate will be used for normalization of gas prices, and for
dual attestation checking.

Script documentation: <code><a href="overview.md#update_exchange_rate">update_exchange_rate</a></code>


---

<a name="@Script_update_minting_ability_43"></a>

#### Script update_minting_ability


Script to allow or disallow minting of new coins in a specified currency.  This transaction can
only be sent by the Treasury Compliance account.  Turning minting off for a currency will have
no effect on coins already in circulation, and coins may still be removed from the system.

Script documentation: <code><a href="overview.md#update_minting_ability">update_minting_ability</a></code>



<a name="@System_Administration_44"></a>

### System Administration


---

<a name="@Script_update_libra_version_45"></a>

#### Script update_libra_version


Updates the Libra major version that is stored on-chain and is used by the VM.  This
transaction can only be sent from the Libra Root account.

Script documentation: <code><a href="overview.md#update_libra_version">update_libra_version</a></code>


---

<a name="@Script_add_to_script_allow_list_46"></a>

#### Script add_to_script_allow_list


Adds a script hash to the transaction allowlist. This transaction
can only be sent by the Libra Root account. Scripts with this hash can be
sent afterward the successful execution of this script.

Script documentation: <code><a href="overview.md#add_to_script_allow_list">add_to_script_allow_list</a></code>



---

<a name="@Transaction_Scripts_47"></a>

## Transaction Scripts

---


<a name="@Account_Creation_48"></a>

### Account Creation



<a name="create_child_vasp_account"></a>

#### Script `create_child_vasp_account`



<a name="@Summary_49"></a>

##### Summary

Creates a Child VASP account with its parent being the sending account of the transaction.
The sender of the transaction must be a Parent VASP account.


<a name="@Technical_Description_50"></a>

##### Technical Description

Creates a <code>ChildVASP</code> account for the sender <code>parent_vasp</code> at <code>child_address</code> with a balance of
<code>child_initial_balance</code> in <code>CoinType</code> and an initial authentication key of
<code>auth_key_prefix | child_address</code>.

If <code>add_all_currencies</code> is true, the child address will have a zero balance in all available
currencies in the system.

The new account will be a child account of the transaction sender, which must be a
Parent VASP account. The child account will be recorded against the limit of
child accounts of the creating Parent VASP account.


<a name="@Events_51"></a>

###### Events

Successful execution with a <code>child_initial_balance</code> greater than zero will emit:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the <code>payer</code> field being the Parent VASP's address,
and payee field being <code>child_address</code>. This is emitted on the Parent VASP's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle.
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> with the  <code>payer</code> field being the Parent VASP's address,
and payee field being <code>child_address</code>. This is emitted on the new Child VASPS's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> handle.


<a name="@Parameters_52"></a>

##### Parameters

| Name                    | Type         | Description                                                                                                                                 |
| ------                  | ------       | -------------                                                                                                                               |
| <code>CoinType</code>              | Type         | The Move type for the <code>CoinType</code> that the child account should be created with. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>parent_vasp</code>           | <code>&signer</code>    | The signer reference of the sending account. Must be a Parent VASP account.                                                                 |
| <code>child_address</code>         | <code>address</code>    | Address of the to-be-created Child VASP account.                                                                                            |
| <code>auth_key_prefix</code>       | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.                                                    |
| <code>add_all_currencies</code>    | <code>bool</code>       | Whether to publish balance resources for all known currencies when the account is created.                                                  |
| <code>child_initial_balance</code> | <code>u64</code>        | The initial balance in <code>CoinType</code> to give the child account when it's created.                                                              |


<a name="@Common_Abort_Conditions_53"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                                             | Description                                                                              |
| ----------------            | --------------                                           | -------------                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>     | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EPARENT_VASP">Roles::EPARENT_VASP</a></code>                                    | The sending account wasn't a Parent VASP account.                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                                        | The <code>child_address</code> address is already taken.                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>    | <code><a href="../../modules/doc/VASP.md#0x1_VASP_ETOO_MANY_CHILDREN">VASP::ETOO_MANY_CHILDREN</a></code>                               | The sending account has reached the maximum number of allowed child accounts.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                                  | The <code>CoinType</code> is not a registered currency on-chain.                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The withdrawal capability for the sending account has already been extracted.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | The sending account doesn't have a balance in <code>CoinType</code>.                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | The sending account doesn't have at least <code>child_initial_balance</code> of <code>CoinType</code> balance. |


<a name="@Related_Scripts_54"></a>

##### Related Scripts

* <code><a href="overview.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="overview.md#add_currency_to_account">Script::add_currency_to_account</a></code>
* <code><a href="overview.md#rotate_authentication_key">Script::rotate_authentication_key</a></code>
* <code><a href="overview.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code>
* <code><a href="overview.md#create_recovery_address">Script::create_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool, child_initial_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_child_vasp_account">LibraAccount::create_child_vasp_account</a>&lt;CoinType&gt;(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    <b>if</b> (child_initial_balance &gt; 0) {
        <b>let</b> vasp_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(parent_vasp);
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;CoinType&gt;(
            &vasp_withdrawal_cap, child_address, child_initial_balance, x"", x""
        );
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(vasp_withdrawal_cap);
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: parent_vasp};
<a name="create_child_vasp_account_parent_addr$1"></a>
<b>let</b> parent_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent_vasp);
<a name="create_child_vasp_account_parent_cap$2"></a>
<b>let</b> parent_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(parent_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountAbortsIf">LibraAccount::CreateChildVASPAccountAbortsIf</a>&lt;CoinType&gt;{
    parent: parent_vasp, new_account_address: child_address};
<b>aborts_if</b> child_initial_balance &gt; max_u64() <b>with</b> <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
<b>include</b> (child_initial_balance &gt; 0) ==&gt;
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: parent_addr};
<b>include</b> (child_initial_balance &gt; 0) ==&gt;
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PayFromAbortsIfRestricted">LibraAccount::PayFromAbortsIfRestricted</a>&lt;CoinType&gt;{
        cap: parent_cap,
        payee: child_address,
        amount: child_initial_balance,
        metadata: x"",
        metadata_signature: x""
    };
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountEnsures">LibraAccount::CreateChildVASPAccountEnsures</a>&lt;CoinType&gt;{
    parent_addr: parent_addr,
    child_addr: child_address,
};
<b>ensures</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;CoinType&gt;(child_address) == child_initial_balance;
<b>ensures</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;CoinType&gt;(parent_addr)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;CoinType&gt;(parent_addr)) - child_initial_balance;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="create_validator_operator_account"></a>

#### Script `create_validator_operator_account`



<a name="@Summary_55"></a>

##### Summary

Creates a Validator Operator account. This transaction can only be sent by the Libra
Root account.


<a name="@Technical_Description_56"></a>

##### Technical Description

Creates an account with a Validator Operator role at <code>new_account_address</code>, with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>. It publishes a
<code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource with the specified <code>human_name</code>.
This script does not assign the validator operator to any validator accounts but only creates the account.


<a name="@Parameters_57"></a>

##### Parameters

| Name                  | Type         | Description                                                                                     |
| ------                | ------       | -------------                                                                                   |
| <code>lr_account</code>          | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
| <code>sliding_nonce</code>       | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                      |
| <code>new_account_address</code> | <code>address</code>    | Address of the to-be-created Validator account.                                                 |
| <code>auth_key_prefix</code>     | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.        |
| <code>human_name</code>          | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator.                                                     |


<a name="@Common_Abort_Conditions_58"></a>

##### Common Abort Conditions

| Error Category | Error Reason | Description |
|----------------|--------------|-------------|
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>          | The sending account is not the Libra Root account or Treasury Compliance account           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>  | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>            | The sending account is not the Libra Root account.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                       | The <code>new_account_address</code> address is already taken.                                        |


<a name="@Related_Scripts_59"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_validator_operator_account">create_validator_operator_account</a>(lr_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_validator_operator_account">create_validator_operator_account</a>(
    lr_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_operator_account">LibraAccount::create_validator_operator_account</a>(
        lr_account,
        new_account_address,
        auth_key_prefix,
        human_name,
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>

Only Libra root may create Validator Operator accounts
Authentication: ValidatorAccountAbortsIf includes AbortsIfNotLibraRoot.
Checks that above table includes all error categories.
The verifier finds an abort that is not documented, and cannot occur in practice:
* REQUIRES_ROLE comes from <code><a href="../../modules/doc/Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a></code>. However, assert_libra_root checks the literal
Libra root address before checking the role, and the role abort is unreachable in practice, since
only Libra root has the Libra root role.


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: lr_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{seq_nonce: sliding_nonce, account: lr_account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountAbortsIf">LibraAccount::CreateValidatorOperatorAccountAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountEnsures">LibraAccount::CreateValidatorOperatorAccountEnsures</a>;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>



</details>

---


<a name="create_validator_account"></a>

#### Script `create_validator_account`



<a name="@Summary_60"></a>

##### Summary

Creates a Validator account. This transaction can only be sent by the Libra
Root account.


<a name="@Technical_Description_61"></a>

##### Technical Description

Creates an account with a Validator role at <code>new_account_address</code>, with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>. It publishes a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource with empty <code>config</code>, and
<code>operator_account</code> fields. The <code>human_name</code> field of the
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is set to the passed in <code>human_name</code>.
This script does not add the validator to the validator set or the system,
but only creates the account.


<a name="@Parameters_62"></a>

##### Parameters

| Name                  | Type         | Description                                                                                     |
| ------                | ------       | -------------                                                                                   |
| <code>lr_account</code>          | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
| <code>sliding_nonce</code>       | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                      |
| <code>new_account_address</code> | <code>address</code>    | Address of the to-be-created Validator account.                                                 |
| <code>auth_key_prefix</code>     | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.        |
| <code>human_name</code>          | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator.                                                     |


<a name="@Common_Abort_Conditions_63"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                            | Description                                                                                |
| ----------------            | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>          | The sending account is not the Libra Root account or Treasury Compliance account           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>  | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>            | The sending account is not the Libra Root account.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                       | The <code>new_account_address</code> address is already taken.                                        |


<a name="@Related_Scripts_64"></a>

##### Related Scripts

* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_validator_account">create_validator_account</a>(lr_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_validator_account">create_validator_account</a>(
    lr_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_account">LibraAccount::create_validator_account</a>(
        lr_account,
        new_account_address,
        auth_key_prefix,
        human_name,
    );
  }
</code></pre>



</details>

<details>
<summary>Specification</summary>

Only Libra root may create Validator accounts
Authentication: ValidatorAccountAbortsIf includes AbortsIfNotLibraRoot.
Checks that above table includes all error categories.
The verifier finds an abort that is not documented, and cannot occur in practice:
* REQUIRES_ROLE comes from <code><a href="../../modules/doc/Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a></code>. However, assert_libra_root checks the literal
Libra root address before checking the role, and the role abort is unreachable in practice, since
only Libra root has the Libra root role.


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: lr_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{seq_nonce: sliding_nonce, account: lr_account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountAbortsIf">LibraAccount::CreateValidatorAccountAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountEnsures">LibraAccount::CreateValidatorAccountEnsures</a>;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>



</details>

---


<a name="create_parent_vasp_account"></a>

#### Script `create_parent_vasp_account`



<a name="@Summary_65"></a>

##### Summary

Creates a Parent VASP account with the specified human name. Must be called by the Treasury Compliance account.


<a name="@Technical_Description_66"></a>

##### Technical Description

Creates an account with the Parent VASP role at <code>address</code> with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code> and a 0 balance of type <code>CoinType</code>. If
<code>add_all_currencies</code> is true, 0 balances for all available currencies in the system will
also be added. This can only be invoked by an TreasuryCompliance account.
<code>sliding_nonce</code> is a unique nonce for operation, see <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> for details.


<a name="@Parameters_67"></a>

##### Parameters

| Name                  | Type         | Description                                                                                                                                                    |
| ------                | ------       | -------------                                                                                                                                                  |
| <code>CoinType</code>            | Type         | The Move type for the <code>CoinType</code> currency that the Parent VASP account should be initialized with. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>          | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                                      |
| <code>sliding_nonce</code>       | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                                                     |
| <code>new_account_address</code> | <code>address</code>    | Address of the to-be-created Parent VASP account.                                                                                                              |
| <code>auth_key_prefix</code>     | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.                                                                       |
| <code>human_name</code>          | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the Parent VASP.                                                                                                                  |
| <code>add_all_currencies</code>  | <code>bool</code>       | Whether to publish balance resources for all known currencies when the account is created.                                                                     |


<a name="@Common_Abort_Conditions_68"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                            | Description                                                                                |
| ----------------            | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>  | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | The sending account is the Treasury Compliance account.                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                 | The <code>CoinType</code> is not a registered currency on-chain.                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                       | The <code>new_account_address</code> address is already taken.                                        |


<a name="@Related_Scripts_69"></a>

##### Related Scripts

* <code><a href="overview.md#create_child_vasp_account">Script::create_child_vasp_account</a></code>
* <code><a href="overview.md#add_currency_to_account">Script::add_currency_to_account</a></code>
* <code><a href="overview.md#rotate_authentication_key">Script::rotate_authentication_key</a></code>
* <code><a href="overview.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code>
* <code><a href="overview.md#create_recovery_address">Script::create_recovery_address</a></code>
* <code><a href="overview.md#rotate_dual_attestation_info">Script::rotate_dual_attestation_info</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_parent_vasp_account">create_parent_vasp_account</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_parent_vasp_account">create_parent_vasp_account</a>&lt;CoinType&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">LibraAccount::create_parent_vasp_account</a>&lt;CoinType&gt;(
        tc_account,
        new_account_address,
        auth_key_prefix,
        human_name,
        add_all_currencies
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: tc_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{account: tc_account, seq_nonce: sliding_nonce};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountAbortsIf">LibraAccount::CreateParentVASPAccountAbortsIf</a>&lt;CoinType&gt;{creator_account: tc_account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountEnsures">LibraAccount::CreateParentVASPAccountEnsures</a>&lt;CoinType&gt;;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>



</details>

---


<a name="create_designated_dealer"></a>

#### Script `create_designated_dealer`



<a name="@Summary_70"></a>

##### Summary

Creates a Designated Dealer account with the provided information, and initializes it with
default mint tiers. The transaction can only be sent by the Treasury Compliance account.


<a name="@Technical_Description_71"></a>

##### Technical Description

Creates an account with the Designated Dealer role at <code>addr</code> with authentication key
<code>auth_key_prefix</code> | <code>addr</code> and a 0 balance of type <code>Currency</code>. If <code>add_all_currencies</code> is true,
0 balances for all available currencies in the system will also be added. This can only be
invoked by an account with the TreasuryCompliance role.

At the time of creation the account is also initialized with default mint tiers of (500_000,
5000_000, 50_000_000, 500_000_000), and preburn areas for each currency that is added to the
account.


<a name="@Parameters_72"></a>

##### Parameters

| Name                 | Type         | Description                                                                                                                                         |
| ------               | ------       | -------------                                                                                                                                       |
| <code>Currency</code>           | Type         | The Move type for the <code>Currency</code> that the Designated Dealer should be initialized with. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>         | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                           |
| <code>sliding_nonce</code>      | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                                          |
| <code>addr</code>               | <code>address</code>    | Address of the to-be-created Designated Dealer account.                                                                                             |
| <code>auth_key_prefix</code>    | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.                                                            |
| <code>human_name</code>         | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the Designated Dealer.                                                                                                 |
| <code>add_all_currencies</code> | <code>bool</code>       | Whether to publish preburn, balance, and tier info resources for all known (SCS) currencies or just <code>Currency</code> when the account is created.         |



<a name="@Common_Abort_Conditions_73"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                            | Description                                                                                |
| ----------------            | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>  | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | The sending account is not the Treasury Compliance account.                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                 | The <code>Currency</code> is not a registered currency on-chain.                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                       | The <code>addr</code> address is already taken.                                                       |


<a name="@Related_Scripts_74"></a>

##### Related Scripts

* <code><a href="overview.md#tiered_mint">Script::tiered_mint</a></code>
* <code><a href="overview.md#peer_to_peer_with_metadata">Script::peer_to_peer_with_metadata</a></code>
* <code><a href="overview.md#rotate_dual_attestation_info">Script::rotate_dual_attestation_info</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_designated_dealer">create_designated_dealer</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, addr: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_designated_dealer">create_designated_dealer</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    addr: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_designated_dealer">LibraAccount::create_designated_dealer</a>&lt;Currency&gt;(
        tc_account,
        addr,
        auth_key_prefix,
        human_name,
        add_all_currencies
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: tc_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{account: tc_account, seq_nonce: sliding_nonce};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerAbortsIf">LibraAccount::CreateDesignatedDealerAbortsIf</a>&lt;Currency&gt;{
    creator_account: tc_account, new_account_address: addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerEnsures">LibraAccount::CreateDesignatedDealerEnsures</a>&lt;Currency&gt;{new_account_address: addr};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>



</details>


---

<a name="@Account_Administration_75"></a>

### Account Administration



<a name="add_currency_to_account"></a>

#### Script `add_currency_to_account`



<a name="@Summary_76"></a>

##### Summary

Adds a zero <code>Currency</code> balance to the sending <code>account</code>. This will enable <code>account</code> to
send, receive, and hold <code><a href="../../modules/doc/Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Currency&gt;</code> coins. This transaction can be
successfully sent by any account that is allowed to hold balances
(e.g., VASP, Designated Dealer).


<a name="@Technical_Description_77"></a>

##### Technical Description

After the successful execution of this transaction the sending account will have a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> resource with zero balance published under it. Only
accounts that can hold balances can send this transaction, the sending account cannot
already have a <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> published under it.


<a name="@Parameters_78"></a>

##### Parameters

| Name       | Type      | Description                                                                                                                                         |
| ------     | ------    | -------------                                                                                                                                       |
| <code>Currency</code> | Type      | The Move type for the <code>Currency</code> being added to the sending account of the transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>account</code>  | <code>&signer</code> | The signer of the sending account of the transaction.                                                                                               |


<a name="@Common_Abort_Conditions_79"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                             | Description                                                                |
| ----------------            | --------------                           | -------------                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                  | The <code>Currency</code> is not a registered currency on-chain.                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">LibraAccount::EROLE_CANT_STORE_BALANCE</a></code> | The sending <code>account</code>'s role does not permit balances.                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">LibraAccount::EADD_EXISTING_CURRENCY</a></code>   | A balance for <code>Currency</code> is already published under the sending <code>account</code>. |


<a name="@Related_Scripts_80"></a>

##### Related Scripts

* <code><a href="overview.md#create_child_vasp_account">Script::create_child_vasp_account</a></code>
* <code><a href="overview.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="overview.md#peer_to_peer_with_metadata">Script::peer_to_peer_with_metadata</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;Currency&gt;(account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">LibraAccount::AddCurrencyAbortsIf</a>&lt;Currency&gt;;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">LibraAccount::AddCurrencyEnsures</a>&lt;Currency&gt;{addr: <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="add_recovery_rotation_capability"></a>

#### Script `add_recovery_rotation_capability`



<a name="@Summary_81"></a>

##### Summary

Stores the sending accounts ability to rotate its authentication key with a designated recovery
account. Both the sending and recovery accounts need to belong to the same VASP and
both be VASP accounts. After this transaction both the sending account and the
specified recovery account can rotate the sender account's authentication key.


<a name="@Technical_Description_82"></a>

##### Technical Description

Adds the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for the sending account
(<code>to_recover_account</code>) to the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource under
<code>recovery_address</code>. After this transaction has been executed successfully the account at
<code>recovery_address</code> and the <code>to_recover_account</code> may rotate the authentication key of
<code>to_recover_account</code> (the sender of this transaction).

The sending account of this transaction (<code>to_recover_account</code>) must not have previously given away its unique key
rotation capability, and must be a VASP account. The account at <code>recovery_address</code>
must also be a VASP account belonging to the same VASP as the <code>to_recover_account</code>.
Additionally the account at <code>recovery_address</code> must have already initialized itself as
a recovery account address using the <code><a href="overview.md#create_recovery_address">Script::create_recovery_address</a></code> transaction script.

The sending account's (<code>to_recover_account</code>) key rotation capability is
removed in this transaction and stored in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code>
resource stored under the account at <code>recovery_address</code>.


<a name="@Parameters_83"></a>

##### Parameters

| Name                 | Type      | Description                                                                                                |
| ------               | ------    | -------------                                                                                              |
| <code>to_recover_account</code> | <code>&signer</code> | The signer reference of the sending account of this transaction.                                           |
| <code>recovery_address</code>   | <code>address</code> | The account address where the <code>to_recover_account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> will be stored. |


<a name="@Common_Abort_Conditions_84"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                                     |
| ----------------           | --------------                                             | -------------                                                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>to_recover_account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>                       | <code>recovery_address</code> does not have a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource published under it.               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION">RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION</a></code>        | <code>to_recover_account</code> and <code>recovery_address</code> do not belong to the same VASP.                     |


<a name="@Related_Scripts_85"></a>

##### Related Scripts

* <code><a href="overview.md#create_recovery_address">Script::create_recovery_address</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#add_recovery_rotation_capability">add_recovery_rotation_capability</a>(to_recover_account: &signer, recovery_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#add_recovery_rotation_capability">add_recovery_rotation_capability</a>(to_recover_account: &signer, recovery_address: address) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability">RecoveryAddress::add_rotation_capability</a>(
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(to_recover_account), recovery_address
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: to_recover_account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>{account: to_recover_account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures">LibraAccount::ExtractKeyRotationCapabilityEnsures</a>{account: to_recover_account};
<a name="add_recovery_rotation_capability_addr$1"></a>
<b>let</b> addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(to_recover_account);
<a name="add_recovery_rotation_capability_rotation_cap$2"></a>
<b>let</b> rotation_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(addr);
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_AddRotationCapabilityAbortsIf">RecoveryAddress::AddRotationCapabilityAbortsIf</a>{
    to_recover: rotation_cap
};
<b>ensures</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(recovery_address)[
    len(<a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(recovery_address)) - 1] == <b>old</b>(rotation_cap);
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="publish_shared_ed25519_public_key"></a>

#### Script `publish_shared_ed25519_public_key`



<a name="@Summary_86"></a>

##### Summary

Rotates the authentication key of the sending account to the
newly-specified public key and publishes a new shared authentication key
under the sender's account. Any account can send this transaction.


<a name="@Technical_Description_87"></a>

##### Technical Description

Rotates the authentication key of the sending account to <code>public_key</code>,
and publishes a <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource
containing the 32-byte ed25519 <code>public_key</code> and the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for
<code>account</code> under <code>account</code>.


<a name="@Parameters_88"></a>

##### Parameters

| Name         | Type         | Description                                                                               |
| ------       | ------       | -------------                                                                             |
| <code>account</code>    | <code>&signer</code>    | The signer reference of the sending account of the transaction.                           |
| <code>public_key</code> | <code>vector&lt;u8&gt;</code> | 32-byte Ed25519 public key for <code>account</code>' authentication key to be rotated to and stored. |


<a name="@Common_Abort_Conditions_89"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                                               | Description                                                                                         |
| ----------------            | --------------                                             | -------------                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> resource.       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_ESHARED_KEY">SharedEd25519PublicKey::ESHARED_KEY</a></code>                      | The <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource is already published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY</a></code>            | <code>public_key</code> is an invalid ed25519 public key.                                                      |


<a name="@Related_Scripts_90"></a>

##### Related Scripts

* <code><a href="overview.md#rotate_shared_ed25519_public_key">Script::rotate_shared_ed25519_public_key</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">SharedEd25519PublicKey::publish</a>(account, public_key)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_PublishAbortsIf">SharedEd25519PublicKey::PublishAbortsIf</a>{key: public_key};
<b>include</b> <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_PublishEnsures">SharedEd25519PublicKey::PublishEnsures</a>{key: public_key};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="create_recovery_address"></a>

#### Script `create_recovery_address`



<a name="@Summary_91"></a>

##### Summary

Initializes the sending account as a recovery address that may be used by
the VASP that it belongs to. The sending account must be a VASP account.
Multiple recovery addresses can exist for a single VASP, but accounts in
each must be disjoint.


<a name="@Technical_Description_92"></a>

##### Technical Description

Publishes a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource under <code>account</code>. It then
extracts the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for <code>account</code> and adds
it to the resource. After the successful execution of this transaction
other accounts may add their key rotation to this resource so that <code>account</code>
may be used as a recovery account for those accounts.


<a name="@Parameters_93"></a>

##### Parameters

| Name      | Type      | Description                                           |
| ------    | ------    | -------------                                         |
| <code>account</code> | <code>&signer</code> | The signer of the sending account of the transaction. |


<a name="@Common_Abort_Conditions_94"></a>

##### Common Abort Conditions

| Error Category              | Error Reason                                               | Description                                                                                   |
| ----------------            | --------------                                             | -------------                                                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">RecoveryAddress::ENOT_A_VASP</a></code>                             | <code>account</code> is not a VASP account.                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">RecoveryAddress::EKEY_ROTATION_DEPENDENCY_CYCLE</a></code>          | A key rotation recovery cycle would be created by adding <code>account</code>'s key rotation capability. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>                       | A <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource has already been published under <code>account</code>.     |


<a name="@Related_Scripts_95"></a>

##### Related Scripts

* <code><a href="overview.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#create_recovery_address">create_recovery_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#create_recovery_address">create_recovery_address</a>(account: &signer) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_publish">RecoveryAddress::publish</a>(account, <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures">LibraAccount::ExtractKeyRotationCapabilityEnsures</a>;
<a name="create_recovery_address_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="create_recovery_address_rotation_cap$2"></a>
<b>let</b> rotation_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_PublishAbortsIf">RecoveryAddress::PublishAbortsIf</a>{
    recovery_account: account,
    rotation_cap: rotation_cap
};
<b>ensures</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">RecoveryAddress::spec_is_recovery_address</a>(account_addr);
<b>ensures</b> len(<a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(account_addr)) == 1;
<b>ensures</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(account_addr)[0] == <b>old</b>(rotation_cap);
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="rotate_authentication_key"></a>

#### Script `rotate_authentication_key`



<a name="@Summary_96"></a>

##### Summary

Rotates the transaction sender's authentication key to the supplied new authentication key. May
be sent by any account.


<a name="@Technical_Description_97"></a>

##### Technical Description

Rotate the <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>authentication_key</code> field to <code>new_key</code>.
<code>new_key</code> must be a valid ed25519 public key, and <code>account</code> must not have previously delegated
its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="@Parameters_98"></a>

##### Parameters

| Name      | Type         | Description                                                 |
| ------    | ------       | -------------                                               |
| <code>account</code> | <code>&signer</code>    | Signer reference of the sending account of the transaction. |
| <code>new_key</code> | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for <code>account</code>.            |


<a name="@Common_Abort_Conditions_99"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                              |
| ----------------           | --------------                                             | -------------                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code>              | <code>new_key</code> was an invalid length.                                                         |


<a name="@Related_Scripts_100"></a>

##### Related Scripts

* <code><a href="overview.md#rotate_authentication_key_with_nonce">Script::rotate_authentication_key_with_nonce</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_nonce_admin">Script::rotate_authentication_key_with_nonce_admin</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;) {
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<a name="rotate_authentication_key_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<a name="rotate_authentication_key_key_rotation_capability$2"></a>
<b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">LibraAccount::RotateAuthenticationKeyAbortsIf</a>{cap: key_rotation_capability, new_authentication_key: new_key};
</code></pre>


This rotates the authentication key of <code>account</code> to <code>new_key</code>


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">LibraAccount::RotateAuthenticationKeyEnsures</a>{addr: account_addr, new_authentication_key: new_key};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="rotate_authentication_key_with_nonce"></a>

#### Script `rotate_authentication_key_with_nonce`



<a name="@Summary_101"></a>

##### Summary

Rotates the sender's authentication key to the supplied new authentication key. May be sent by
any account that has a sliding nonce resource published under it (usually this is Treasury
Compliance or Libra Root accounts).


<a name="@Technical_Description_102"></a>

##### Technical Description

Rotates the <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>authentication_key</code> field to <code>new_key</code>.
<code>new_key</code> must be a valid ed25519 public key, and <code>account</code> must not have previously delegated
its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="@Parameters_103"></a>

##### Parameters

| Name            | Type         | Description                                                                |
| ------          | ------       | -------------                                                              |
| <code>account</code>       | <code>&signer</code>    | Signer reference of the sending account of the transaction.                |
| <code>sliding_nonce</code> | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction. |
| <code>new_key</code>       | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for <code>account</code>.                           |


<a name="@Common_Abort_Conditions_104"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                                |
| ----------------           | --------------                                             | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                             | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                             | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>                    | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code>              | <code>new_key</code> was an invalid length.                                                           |


<a name="@Related_Scripts_105"></a>

##### Related Scripts

* <code><a href="overview.md#rotate_authentication_key">Script::rotate_authentication_key</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_nonce_admin">Script::rotate_authentication_key_with_nonce_admin</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a>(account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a>(account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<a name="rotate_authentication_key_with_nonce_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<a name="rotate_authentication_key_with_nonce_key_rotation_capability$2"></a>
<b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">LibraAccount::RotateAuthenticationKeyAbortsIf</a>{cap: key_rotation_capability, new_authentication_key: new_key};
</code></pre>


This rotates the authentication key of <code>account</code> to <code>new_key</code>


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">LibraAccount::RotateAuthenticationKeyEnsures</a>{addr: account_addr, new_authentication_key: new_key};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="rotate_authentication_key_with_nonce_admin"></a>

#### Script `rotate_authentication_key_with_nonce_admin`



<a name="@Summary_106"></a>

##### Summary

Rotates the specified account's authentication key to the supplied new authentication key. May
only be sent by the Libra Root account as a write set transaction.


<a name="@Technical_Description_107"></a>

##### Technical Description

Rotate the <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>authentication_key</code> field to <code>new_key</code>.
<code>new_key</code> must be a valid ed25519 public key, and <code>account</code> must not have previously delegated
its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="@Parameters_108"></a>

##### Parameters

| Name            | Type         | Description                                                                                                  |
| ------          | ------       | -------------                                                                                                |
| <code>lr_account</code>    | <code>&signer</code>    | The signer reference of the sending account of the write set transaction. May only be the Libra Root signer. |
| <code>account</code>       | <code>&signer</code>    | Signer reference of account specified in the <code>execute_as</code> field of the write set transaction.                |
| <code>sliding_nonce</code> | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction for Libra Root.                    |
| <code>new_key</code>       | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for <code>account</code>.                                                             |


<a name="@Common_Abort_Conditions_109"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                                                |
| ----------------           | --------------                                             | -------------                                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                             | The <code>sliding_nonce</code> in <code>lr_account</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                             | The <code>sliding_nonce</code> in <code>lr_account</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>                    | The <code>sliding_nonce</code> in<code> lr_account</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code>              | <code>new_key</code> was an invalid length.                                                                           |


<a name="@Related_Scripts_110"></a>

##### Related Scripts

* <code><a href="overview.md#rotate_authentication_key">Script::rotate_authentication_key</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_nonce">Script::rotate_authentication_key_with_nonce</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<a name="rotate_authentication_key_with_nonce_admin_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ account: lr_account, seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<a name="rotate_authentication_key_with_nonce_admin_key_rotation_capability$2"></a>
<b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">LibraAccount::RotateAuthenticationKeyAbortsIf</a>{cap: key_rotation_capability, new_authentication_key: new_key};
</code></pre>


This rotates the authentication key of <code>account</code> to <code>new_key</code>


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">LibraAccount::RotateAuthenticationKeyEnsures</a>{addr: account_addr, new_authentication_key: new_key};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="rotate_authentication_key_with_recovery_address"></a>

#### Script `rotate_authentication_key_with_recovery_address`



<a name="@Summary_111"></a>

##### Summary

Rotates the authentication key of a specified account that is part of a recovery address to a
new authentication key. Only used for accounts that are part of a recovery address (see
<code><a href="overview.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code> for account restrictions).


<a name="@Technical_Description_112"></a>

##### Technical Description

Rotates the authentication key of the <code>to_recover</code> account to <code>new_key</code> using the
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> stored in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource
published under <code>recovery_address</code>. This transaction can be sent either by the <code>to_recover</code>
account, or by the account where the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource is published
that contains <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="@Parameters_113"></a>

##### Parameters

| Name               | Type         | Description                                                                                                                    |
| ------             | ------       | -------------                                                                                                                  |
| <code>account</code>          | <code>&signer</code>    | Signer reference of the sending account of the transaction.                                                                    |
| <code>recovery_address</code> | <code>address</code>    | Address where <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> that holds <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> is published. |
| <code>to_recover</code>       | <code>address</code>    | The address of the account whose authentication key will be updated.                                                           |
| <code>new_key</code>          | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for the account at the <code>to_recover</code> address.                                                 |


<a name="@Common_Abort_Conditions_114"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                  | Description                                                                                                                                          |
| ----------------           | --------------                                | -------------                                                                                                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>          | <code>recovery_address</code> does not have a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource published under it.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY">RecoveryAddress::ECANNOT_ROTATE_KEY</a></code>         | The address of <code>account</code> is not <code>recovery_address</code> or <code>to_recover</code>.                                                                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE">RecoveryAddress::EACCOUNT_NOT_RECOVERABLE</a></code>   | <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>  is not in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code>  resource published under <code>recovery_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code> | <code>new_key</code> was an invalid length.                                                                                                                     |


<a name="@Related_Scripts_115"></a>

##### Related Scripts

* <code><a href="overview.md#rotate_authentication_key">Script::rotate_authentication_key</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_nonce">Script::rotate_authentication_key_with_nonce</a></code>
* <code><a href="overview.md#rotate_authentication_key_with_nonce_admin">Script::rotate_authentication_key_with_nonce_admin</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(
    account: &signer,
    recovery_address: address,
    to_recover: address,
    new_key: vector&lt;u8&gt;
) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">RecoveryAddress::rotate_authentication_key</a>(account, recovery_address, to_recover, new_key)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyAbortsIf">RecoveryAddress::RotateAuthenticationKeyAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyEnsures">RecoveryAddress::RotateAuthenticationKeyEnsures</a>;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="rotate_dual_attestation_info"></a>

#### Script `rotate_dual_attestation_info`



<a name="@Summary_116"></a>

##### Summary

Updates the url used for off-chain communication, and the public key used to verify dual
attestation on-chain. Transaction can be sent by any account that has dual attestation
information published under it. In practice the only such accounts are Designated Dealers and
Parent VASPs.


<a name="@Technical_Description_117"></a>

##### Technical Description

Updates the <code>base_url</code> and <code>compliance_public_key</code> fields of the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code>
resource published under <code>account</code>. The <code>new_key</code> must be a valid ed25519 public key.


<a name="@Events_118"></a>

###### Events

Successful execution of this transaction emits two events:
* A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_ComplianceKeyRotationEvent">DualAttestation::ComplianceKeyRotationEvent</a></code> containing the new compliance public key, and
the blockchain time at which the key was updated emitted on the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code>
<code>compliance_key_rotation_events</code> handle published under <code>account</code>; and
* A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_BaseUrlRotationEvent">DualAttestation::BaseUrlRotationEvent</a></code> containing the new base url to be used for
off-chain communication, and the blockchain time at which the url was updated emitted on the
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> <code>base_url_rotation_events</code> handle published under <code>account</code>.


<a name="@Parameters_119"></a>

##### Parameters

| Name      | Type         | Description                                                               |
| ------    | ------       | -------------                                                             |
| <code>account</code> | <code>&signer</code>    | Signer reference of the sending account of the transaction.               |
| <code>new_url</code> | <code>vector&lt;u8&gt;</code> | ASCII-encoded url to be used for off-chain communication with <code>account</code>.  |
| <code>new_key</code> | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for on-chain dual attestation checking. |


<a name="@Common_Abort_Conditions_120"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                           | Description                                                                |
| ----------------           | --------------                         | -------------                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_ECREDENTIAL">DualAttestation::ECREDENTIAL</a></code>         | A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> resource is not published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EINVALID_PUBLIC_KEY">DualAttestation::EINVALID_PUBLIC_KEY</a></code> | <code>new_key</code> is not a valid ed25519 public key.                               |


<a name="@Related_Scripts_121"></a>

##### Related Scripts

* <code><a href="overview.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="overview.md#create_designated_dealer">Script::create_designated_dealer</a></code>
* <code><a href="overview.md#rotate_dual_attestation_info">Script::rotate_dual_attestation_info</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_dual_attestation_info">rotate_dual_attestation_info</a>(account: &signer, new_url: vector&lt;u8&gt;, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_dual_attestation_info">rotate_dual_attestation_info</a>(account: &signer, new_url: vector&lt;u8&gt;, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_rotate_base_url">DualAttestation::rotate_base_url</a>(account, new_url);
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_rotate_compliance_public_key">DualAttestation::rotate_compliance_public_key</a>(account, new_key)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_RotateBaseUrlAbortsIf">DualAttestation::RotateBaseUrlAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_RotateBaseUrlEnsures">DualAttestation::RotateBaseUrlEnsures</a>;
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_RotateCompliancePublicKeyAbortsIf">DualAttestation::RotateCompliancePublicKeyAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_RotateCompliancePublicKeyEnsures">DualAttestation::RotateCompliancePublicKeyEnsures</a>;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="rotate_shared_ed25519_public_key"></a>

#### Script `rotate_shared_ed25519_public_key`



<a name="@Summary_122"></a>

##### Summary

Rotates the authentication key in a <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code>. This transaction can be sent by
any account that has previously published a shared ed25519 public key using
<code><a href="overview.md#publish_shared_ed25519_public_key">Script::publish_shared_ed25519_public_key</a></code>.


<a name="@Technical_Description_123"></a>

##### Technical Description

This first rotates the public key stored in <code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource to <code>public_key</code>, after which it
rotates the authentication key using the capability stored in <code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> to a new value derived from <code>public_key</code>


<a name="@Parameters_124"></a>

##### Parameters

| Name         | Type         | Description                                                     |
| ------       | ------       | -------------                                                   |
| <code>account</code>    | <code>&signer</code>    | The signer reference of the sending account of the transaction. |
| <code>public_key</code> | <code>vector&lt;u8&gt;</code> | 32-byte Ed25519 public key.                                     |


<a name="@Common_Abort_Conditions_125"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                    | Description                                                                                   |
| ----------------           | --------------                                  | -------------                                                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_ESHARED_KEY">SharedEd25519PublicKey::ESHARED_KEY</a></code>           | A <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource is not published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY</a></code> | <code>public_key</code> is an invalid ed25519 public key.                                                |


<a name="@Related_Scripts_126"></a>

##### Related Scripts

* <code><a href="overview.md#publish_shared_ed25519_public_key">Script::publish_shared_ed25519_public_key</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">SharedEd25519PublicKey::rotate_key</a>(account, public_key)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_RotateKeyAbortsIf">SharedEd25519PublicKey::RotateKeyAbortsIf</a>{new_public_key: public_key};
<b>include</b> <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_RotateKeyEnsures">SharedEd25519PublicKey::RotateKeyEnsures</a>{new_public_key: public_key};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

---


<a name="mint_lbr"></a>

#### Script `mint_lbr`



<a name="@Summary_127"></a>

##### Summary

Mints LBR from the sending account's constituent coins by depositing in the
on-chain LBR reserve. Deposits the newly-minted LBR into the sending
account. Can be sent by any account that can hold balances for the constituent
currencies for LBR and LBR.


<a name="@Technical_Description_128"></a>

##### Technical Description

Mints <code>amount_lbr</code> LBR from the sending account's constituent coins and deposits the
resulting LBR into the sending account.


<a name="@Events_129"></a>

###### Events

Successful execution of this script emits three events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin1 currency code, and a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin2 currency code on <code>account</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle with the <code>amounts</code> for each event being the
components amounts of <code>amount_lbr</code> LBR; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code>
<code>received_events</code> handle with the LBR currency code and amount field equal to <code>amount_lbr</code>.


<a name="@Parameters_130"></a>

##### Parameters

| Name         | Type      | Description                                      |
| ------       | ------    | -------------                                    |
| <code>account</code>    | <code>&signer</code> | The signer reference of the sending account.     |
| <code>amount_lbr</code> | <code>u64</code>     | The amount of LBR (in microlibra) to be created. |


<a name="@Common_Abort_Conditions_131"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                     | Description                                                                      |
| ----------------           | --------------                                   | -------------                                                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>      | <code>account</code> doesn't hold a balance in one of the backing currencies of LBR.        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LBR.md#0x1_LBR_EZERO_LBR_MINT_NOT_ALLOWED">LBR::EZERO_LBR_MINT_NOT_ALLOWED</a></code>                | <code>amount_lbr</code> passed in was zero.                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LBR.md#0x1_LBR_ECOIN1">LBR::ECOIN1</a></code>                                    | The amount of <code><a href="../../modules/doc/Coin1.md#0x1_Coin1">Coin1</a></code> needed for the specified LBR would exceed <code><a href="../../modules/doc/LBR.md#0x1_LBR_MAX_U64">LBR::MAX_U64</a></code>.  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LBR.md#0x1_LBR_ECOIN2">LBR::ECOIN2</a></code>                                    | The amount of <code><a href="../../modules/doc/Coin2.md#0x1_Coin2">Coin2</a></code> needed for the specified LBR would exceed <code><a href="../../modules/doc/LBR.md#0x1_LBR_MAX_U64">LBR::MAX_U64</a></code>.  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINTING_NOT_ALLOWED">Libra::EMINTING_NOT_ALLOWED</a></code>                    | Minting of LBR is not allowed currently.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | <code>account</code> doesn't hold a balance in LBR.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>       | <code>account</code> has exceeded its daily withdrawal limits for the backing coins of LBR. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>          | <code>account</code> has exceeded its daily deposit limits for LBR.                         |


<a name="@Related_Scripts_132"></a>

##### Related Scripts

* <code><a href="overview.md#unmint_lbr">Script::unmint_lbr</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_staple_lbr">LibraAccount::staple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<a name="mint_lbr_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="mint_lbr_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">LibraAccount::StapleLBRAbortsIf</a>{cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">LibraAccount::StapleLBREnsures</a>{cap: cap};
</code></pre>



</details>

---


<a name="unmint_lbr"></a>

#### Script `unmint_lbr`



<a name="@Summary_133"></a>

##### Summary

Withdraws a specified amount of LBR from the transaction sender's account, and unstaples the
withdrawn LBR into its constituent coins. Deposits each of the constituent coins to the
transaction sender's balances. Any account that can hold balances that has the correct balances
may send this transaction.


<a name="@Technical_Description_134"></a>

##### Technical Description

Withdraws <code>amount_lbr</code> LBR coins from the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;<a href="../../modules/doc/LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;</code> balance held under
<code>account</code>. Withdraws the backing coins for the LBR coins from the on-chain reserve in the
<code><a href="../../modules/doc/LBR.md#0x1_LBR_Reserve">LBR::Reserve</a></code> resource published under <code>0xA550C18</code>. It then deposits each of the backing coins
into balance resources published under <code>account</code>.


<a name="@Events_135"></a>

###### Events

Successful execution of this transaction will emit two <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code>s. One
for each constituent currency that is unstapled and returned to the sending <code>account</code>'s
balances.


<a name="@Parameters_136"></a>

##### Parameters

| Name         | Type      | Description                                                     |
| ------       | ------    | -------------                                                   |
| <code>account</code>    | <code>&signer</code> | The signer reference of the sending account of the transaction. |
| <code>amount_lbr</code> | <code>u64</code>     | The amount of microlibra to unstaple.                           |


<a name="@Common_Abort_Conditions_137"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                             | Description                                                                               |
| ----------------           | --------------                                           | -------------                                                                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a></code> for <code>account</code> has previously been extracted.       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | <code>account</code> doesn't have a balance in LBR.                                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | <code>amount_lbr</code> is greater than the balance of LBR in <code>account</code>.                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECOIN">Libra::ECOIN</a></code>                                           | <code>amount_lbr</code> is zero.                                                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>               | <code>account</code> has exceeded its daily withdrawal limits for LBR.                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>                  | <code>account</code> has exceeded its daily deposit limits for one of the backing currencies of LBR. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code>         | <code>account</code> doesn't hold a balance in one or both of the backing currencies of LBR.         |


<a name="@Related_Scripts_138"></a>

##### Related Scripts

* <code><a href="overview.md#mint_lbr">Script::mint_lbr</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_unstaple_lbr">LibraAccount::unstaple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
}
</code></pre>



</details>


---

<a name="@Payments_139"></a>

### Payments



<a name="peer_to_peer_with_metadata"></a>

#### Script `peer_to_peer_with_metadata`



<a name="@Summary_140"></a>

##### Summary

Transfers a given number of coins in a specified currency from one account to another.
Transfers over a specified amount defined on-chain that are between two different VASPs, or
other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
agreed to receive the coins.  This transaction can be sent by any account that can hold a
balance, and to any account that can hold a balance. Both accounts must hold balances in the
currency being transacted.


<a name="@Technical_Description_141"></a>

##### Technical Description


Transfers <code>amount</code> coins of type <code>Currency</code> from <code>payer</code> to <code>payee</code> with (optional) associated
<code>metadata</code> and an (optional) <code>metadata_signature</code> on the message
<code>metadata</code> | <code><a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(payer)</code> | <code>amount</code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_DOMAIN_SEPARATOR">DualAttestation::DOMAIN_SEPARATOR</a></code>.
The <code>metadata</code> and <code>metadata_signature</code> parameters are only required if <code>amount</code> >=
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_get_cur_microlibra_limit">DualAttestation::get_cur_microlibra_limit</a></code> LBR and <code>payer</code> and <code>payee</code> are distinct VASPs.
However, a transaction sender can opt in to dual attestation even when it is not required
(e.g., a DesignatedDealer -> VASP payment) by providing a non-empty <code>metadata_signature</code>.
Standardized <code>metadata</code> LCS format can be found in <code>libra_types::transaction::metadata::Metadata</code>.


<a name="@Events_142"></a>

###### Events

Successful execution of this script emits two events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> on <code>payer</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>payee</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> handle.


<a name="@Parameters_143"></a>

##### Parameters

| Name                 | Type         | Description                                                                                                                  |
| ------               | ------       | -------------                                                                                                                |
| <code>Currency</code>           | Type         | The Move type for the <code>Currency</code> being sent in this transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>payer</code>              | <code>&signer</code>    | The signer reference of the sending account that coins are being transferred from.                                           |
| <code>payee</code>              | <code>address</code>    | The address of the account the coins are being transferred to.                                                               |
| <code>metadata</code>           | <code>vector&lt;u8&gt;</code> | Optional metadata about this payment.                                                                                        |
| <code>metadata_signature</code> | <code>vector&lt;u8&gt;</code> | Optional signature over <code>metadata</code> and payment information. See                                                              |


<a name="@Common_Abort_Conditions_144"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                     | Description                                                                                                                         |
| ----------------           | --------------                                   | -------------                                                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>      | <code>payer</code> doesn't hold a balance in <code>Currency</code>.                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>            | <code>amount</code> is greater than <code>payer</code>'s balance in <code>Currency</code>.                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">LibraAccount::ECOIN_DEPOSIT_IS_ZERO</a></code>            | <code>amount</code> is zero.                                                                                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST">LibraAccount::EPAYEE_DOES_NOT_EXIST</a></code>            | No account exists at the <code>payee</code> address.                                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | An account exists at <code>payee</code>, but it does not accept payments in <code>Currency</code>.                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_EACCOUNT_FROZEN">AccountFreezing::EACCOUNT_FROZEN</a></code>               | The <code>payee</code> account is frozen.                                                                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EMALFORMED_METADATA_SIGNATURE">DualAttestation::EMALFORMED_METADATA_SIGNATURE</a></code> | <code>metadata_signature</code> is not 64 bytes.                                                                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EINVALID_METADATA_SIGNATURE">DualAttestation::EINVALID_METADATA_SIGNATURE</a></code>   | <code>metadata_signature</code> does not verify on the against the <code>payee'</code>s <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> <code>compliance_public_key</code> public key. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>       | <code>payer</code> has exceeded its daily withdrawal limits for the backing coins of LBR.                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>          | <code>payee</code> has exceeded its daily deposit limits for LBR.                                                                              |


<a name="@Related_Scripts_145"></a>

##### Related Scripts

* <code><a href="overview.md#create_child_vasp_account">Script::create_child_vasp_account</a></code>
* <code><a href="overview.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="overview.md#add_currency_to_account">Script::add_currency_to_account</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
    <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;Currency&gt;(
        &payer_withdrawal_cap, payee, amount, metadata, metadata_signature
    );
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: payer};
<a name="peer_to_peer_with_metadata_payer_addr$1"></a>
<b>let</b> payer_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer);
<a name="peer_to_peer_with_metadata_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(payer_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: payer_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PayFromAbortsIf">LibraAccount::PayFromAbortsIf</a>&lt;Currency&gt;{cap: cap};
</code></pre>


The balances of payer and payee change by the correct amount.


<pre><code><b>ensures</b> payer_addr != payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payer_addr)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payer_addr)) - amount;
<b>ensures</b> payer_addr != payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)) + amount;
<b>ensures</b> payer_addr == payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee));
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
</code></pre>



</details>


---

<a name="@Validator_and_Validator_Operator_Administration_146"></a>

### Validator and Validator Operator Administration



<a name="add_validator_and_reconfigure"></a>

#### Script `add_validator_and_reconfigure`



<a name="@Summary_147"></a>

##### Summary

Adds a validator account to the validator set, and triggers a
reconfiguration of the system to admit the account to the validator set for the system. This
transaction can only be successfully called by the Libra Root account.


<a name="@Technical_Description_148"></a>

##### Technical Description

This script adds the account at <code>validator_address</code> to the validator set.
This transaction emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> event and triggers a
reconfiguration. Once the reconfiguration triggered by this script's
execution has been performed, the account at the <code>validator_address</code> is
considered to be a validator in the network.

This transaction script will fail if the <code>validator_address</code> address is already in the validator set
or does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource already published under it.


<a name="@Parameters_149"></a>

##### Parameters

| Name                | Type         | Description                                                                                                                        |
| ------              | ------       | -------------                                                                                                                      |
| <code>lr_account</code>        | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer.                                    |
| <code>sliding_nonce</code>     | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                         |
| <code>validator_name</code>    | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator. Must match the human name in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> for the validator. |
| <code>validator_address</code> | <code>address</code>    | The validator account address to be added to the validator set.                                                                    |


<a name="@Common_Abort_Conditions_150"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                  | Description                                                                                                                               |
| ----------------           | --------------                                | -------------                                                                                                                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>                  | The sending account is not the Libra Root account.                                                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                | The <code>sliding_nonce</code> is too far in the future.                                                                                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>       | The <code>sliding_nonce</code> has been previously recorded.                                                                                         |
| 0                          | 0                                             | The provided <code>validator_name</code> does not match the already-recorded human name for the validator.                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR">LibraSystem::EINVALID_PROSPECTIVE_VALIDATOR</a></code> | The validator to be added does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it, or its <code>config</code> field is empty. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_EALREADY_A_VALIDATOR">LibraSystem::EALREADY_A_VALIDATOR</a></code>           | The <code>validator_address</code> account is already a registered validator.                                                                        |


<a name="@Related_Scripts_151"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#add_validator_and_reconfigure">add_validator_and_reconfigure</a>(lr_account: &signer, sliding_nonce: u64, validator_name: vector&lt;u8&gt;, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#add_validator_and_reconfigure">add_validator_and_reconfigure</a>(
    lr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector&lt;u8&gt;,
    validator_address: address
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>assert</b>(<a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) == validator_name, 0);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(lr_account, validator_address);
}
</code></pre>



</details>

---


<a name="register_validator_config"></a>

#### Script `register_validator_config`



<a name="@Summary_152"></a>

##### Summary

Updates a validator's configuration. This does not reconfigure the system and will not update
the configuration in the validator set that is seen by other validators in the network. Can
only be successfully sent by a Validator Operator account that is already registered with a
validator.


<a name="@Technical_Description_153"></a>

##### Technical Description

This updates the fields with corresponding names held in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It does not emit a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code>
so the copy of this config held in the validator set will not be updated, and the changes are
only "locally" under the <code>validator_account</code> account address.


<a name="@Parameters_154"></a>

##### Parameters

| Name                          | Type         | Description                                                                                                                  |
| ------                        | ------       | -------------                                                                                                                |
| <code>validator_operator_account</code>  | <code>&signer</code>    | Signer reference of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                                    |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                                         |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                       |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                        |


<a name="@Common_Abort_Conditions_155"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |


<a name="@Related_Scripts_156"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#register_validator_config">register_validator_config</a>(validator_operator_account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#register_validator_config">register_validator_config</a>(
    validator_operator_account: &signer,
    // TODO Rename <b>to</b> validator_addr, since it is an address.
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        validator_operator_account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses
    );
 }
</code></pre>



</details>

<details>
<summary>Specification</summary>

Access control rule is that only the validator operator for a validator may set
call this, but there is an aborts_if in SetConfigAbortsIf that tests that directly.


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: validator_operator_account};
<b>include</b> <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_SetConfigAbortsIf">ValidatorConfig::SetConfigAbortsIf</a> {validator_addr: validator_account};
<b>ensures</b> <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_account);
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="remove_validator_and_reconfigure"></a>

#### Script `remove_validator_and_reconfigure`



<a name="@Summary_157"></a>

##### Summary

This script removes a validator account from the validator set, and triggers a reconfiguration
of the system to remove the validator from the system. This transaction can only be
successfully called by the Libra Root account.


<a name="@Technical_Description_158"></a>

##### Technical Description

This script removes the account at <code>validator_address</code> from the validator set. This transaction
emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> event. Once the reconfiguration triggered by this event
has been performed, the account at <code>validator_address</code> is no longer considered to be a
validator in the network. This transaction will fail if the validator at <code>validator_address</code>
is not in the validator set.


<a name="@Parameters_159"></a>

##### Parameters

| Name                | Type         | Description                                                                                                                        |
| ------              | ------       | -------------                                                                                                                      |
| <code>lr_account</code>        | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer.                                    |
| <code>sliding_nonce</code>     | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                         |
| <code>validator_name</code>    | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator. Must match the human name in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> for the validator. |
| <code>validator_address</code> | <code>address</code>    | The validator account address to be removed from the validator set.                                                                |


<a name="@Common_Abort_Conditions_160"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                     |
| ----------------           | --------------                          | -------------                                                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                               |
| 0                          | 0                                       | The provided <code>validator_name</code> does not match the already-recorded human name for the validator. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">LibraSystem::ENOT_AN_ACTIVE_VALIDATOR</a></code> | The validator to be removed is not in the validator set.                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>            | The sending account is not the Libra Root account.                                              |


<a name="@Related_Scripts_161"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#remove_validator_and_reconfigure">remove_validator_and_reconfigure</a>(lr_account: &signer, sliding_nonce: u64, validator_name: vector&lt;u8&gt;, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#remove_validator_and_reconfigure">remove_validator_and_reconfigure</a>(
    lr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector&lt;u8&gt;,
    validator_address: address
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>assert</b>(<a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) == validator_name, 0);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_remove_validator">LibraSystem::remove_validator</a>(lr_account, validator_address);
}
</code></pre>



</details>

---


<a name="set_validator_config_and_reconfigure"></a>

#### Script `set_validator_config_and_reconfigure`



<a name="@Summary_162"></a>

##### Summary

Updates a validator's configuration, and triggers a reconfiguration of the system to update the
validator set with this new validator configuration.  Can only be successfully sent by a
Validator Operator account that is already registered with a validator.


<a name="@Technical_Description_163"></a>

##### Technical Description

This updates the fields with corresponding names held in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It then emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> to
trigger a reconfiguration of the system.  This reconfiguration will update the validator set
on-chain with the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.


<a name="@Parameters_164"></a>

##### Parameters

| Name                          | Type         | Description                                                                                                                  |
| ------                        | ------       | -------------                                                                                                                |
| <code>validator_operator_account</code>  | <code>&signer</code>    | Signer reference of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                                    |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                                         |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                       |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                        |


<a name="@Common_Abort_Conditions_165"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |


<a name="@Related_Scripts_166"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(validator_operator_account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(
    validator_operator_account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        validator_operator_account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses
    );
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_update_config_and_reconfigure">LibraSystem::update_config_and_reconfigure</a>(validator_operator_account, validator_account);
 }
</code></pre>



</details>

---


<a name="set_validator_operator"></a>

#### Script `set_validator_operator`



<a name="@Summary_167"></a>

##### Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by an account with
Validator role.


<a name="@Technical_Description_168"></a>

##### Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the sending validator account. The account at <code>operator_account</code> address must have
a Validator Operator role and have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code>
resource published under it. The sending <code>account</code> must be a Validator and have a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. This script does not emit a
<code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> and no reconfiguration of the system is initiated by this script.


<a name="@Parameters_169"></a>

##### Parameters

| Name               | Type         | Description                                                                                  |
| ------             | ------       | -------------                                                                                |
| <code>account</code>          | <code>&signer</code>    | The signer reference of the sending account of the transaction.                              |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                             |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator. |


<a name="@Common_Abort_Conditions_170"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| 0                          | 0                                                     | The <code>human_name</code> field of the <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="@Related_Scripts_171"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#set_validator_operator">set_validator_operator</a>(account: &signer, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#set_validator_operator">set_validator_operator</a>(
    account: &signer,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <b>assert</b>(<a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(account, operator_account);
}
</code></pre>



</details>

---


<a name="set_validator_operator_with_nonce_admin"></a>

#### Script `set_validator_operator_with_nonce_admin`



<a name="@Summary_172"></a>

##### Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by the Libra Root
account as a write set transaction.


<a name="@Technical_Description_173"></a>

##### Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the validator <code>account</code>. The account at <code>operator_account</code> address must have a
Validator Operator role and have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource
published under it. The account represented by the <code>account</code> signer must be a Validator and
have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. No reconfiguration of
the system is initiated by this script.


<a name="@Parameters_174"></a>

##### Parameters

| Name               | Type         | Description                                                                                                  |
| ------             | ------       | -------------                                                                                                |
| <code>lr_account</code>       | <code>&signer</code>    | The signer reference of the sending account of the write set transaction. May only be the Libra Root signer. |
| <code>account</code>          | <code>&signer</code>    | Signer reference of account specified in the <code>execute_as</code> field of the write set transaction.                |
| <code>sliding_nonce</code>    | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction for Libra Root.                    |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                                             |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator.                 |


<a name="@Common_Abort_Conditions_175"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                        | The <code>sliding_nonce</code> in <code>lr_account</code> is too old and it's impossible to determine if it's duplicated or not.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                        | The <code>sliding_nonce</code> in <code>lr_account</code> is too far in the future.                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>               | The <code>sliding_nonce</code> in<code> lr_account</code> has been previously recorded.                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| 0                          | 0                                                     | The <code>human_name</code> field of the <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="@Related_Scripts_176"></a>

##### Related Scripts

* <code><a href="overview.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="overview.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="overview.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="overview.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="overview.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="overview.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="overview.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(
    lr_account: &signer,
    account: &signer,
    sliding_nonce: u64,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>assert</b>(<a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(account, operator_account);
}
</code></pre>



</details>


---

<a name="@Treasury_and_Compliance_Operations_177"></a>

### Treasury and Compliance Operations



<a name="preburn"></a>

#### Script `preburn`



<a name="@Summary_178"></a>

##### Summary

Moves a specified number of coins in a given currency from the account's
balance to its preburn area after which the coins may be burned. This
transaction may be sent by any account that holds a balance and preburn area
in the specified currency.


<a name="@Technical_Description_179"></a>

##### Technical Description

Moves the specified <code>amount</code> of coins in <code>Token</code> currency from the sending <code>account</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code> to the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> published under the same
<code>account</code>. <code>account</code> must have both of these resources published under it at the start of this
transaction in order for it to execute successfully.


<a name="@Events_180"></a>

###### Events

Successful execution of this script emits two events:
* <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a> </code> on <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code>
handle with the <code>payee</code> and <code>payer</code> fields being <code>account</code>'s address; and
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_PreburnEvent">Libra::PreburnEvent</a></code> with <code>Token</code>'s currency code on the
<code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token</code>'s <code>preburn_events</code> handle for <code>Token</code> and with
<code>preburn_address</code> set to <code>account</code>'s address.


<a name="@Parameters_181"></a>

##### Parameters

| Name      | Type      | Description                                                                                                                      |
| ------    | ------    | -------------                                                                                                                    |
| <code>Token</code>   | Type      | The Move type for the <code>Token</code> currency being moved to the preburn area. <code>Token</code> must be an already-registered currency on-chain. |
| <code>account</code> | <code>&signer</code> | The signer reference of the sending account.                                                                                     |
| <code>amount</code>  | <code>u64</code>     | The amount in <code>Token</code> to be moved to the preburn area.                                                                           |


<a name="@Common_Abort_Conditions_182"></a>

##### Common Abort Conditions

| Error Category           | Error Reason                                             | Description                                                                             |
| ----------------         | --------------                                           | -------------                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                                  | The <code>Token</code> is not a registered currency on-chain.                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The withdrawal capability for <code>account</code> has already been extracted.                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | <code>amount</code> is greater than <code>payer</code>'s balance in <code>Token</code>.                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | <code>account</code> doesn't hold a balance in <code>Token</code>.                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                                        | <code>account</code> doesn't have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it.           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN_OCCUPIED">Libra::EPREBURN_OCCUPIED</a></code>                               | The <code>value</code> field in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource under the sender is non-zero. |


<a name="@Related_Scripts_183"></a>

##### Related Scripts

* <code><a href="overview.md#cancel_burn">Script::cancel_burn</a></code>
* <code><a href="overview.md#burn">Script::burn</a></code>
* <code><a href="overview.md#burn_txn_fees">Script::burn_txn_fees</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_preburn">LibraAccount::preburn</a>&lt;Token&gt;(account, &withdraw_cap, amount);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<a name="preburn_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="preburn_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PreburnAbortsIf">LibraAccount::PreburnAbortsIf</a>&lt;Token&gt;{dd: account, cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PreburnEnsures">LibraAccount::PreburnEnsures</a>&lt;Token&gt;{dd_addr: account_addr, payer: account_addr};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
</code></pre>



</details>

---


<a name="burn"></a>

#### Script `burn`



<a name="@Summary_184"></a>

##### Summary

Burns all coins held in the preburn resource at the specified
preburn address and removes them from the system. The sending account must
be the Treasury Compliance account.
The account that holds the preburn resource will normally be a Designated
Dealer, but there are no enforced requirements that it be one.


<a name="@Technical_Description_185"></a>

##### Technical Description

This transaction permanently destroys all the coins of <code>Token</code> type
stored in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under the
<code>preburn_address</code> account address.

This transaction will only succeed if the sending <code>account</code> has a
<code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code>, and a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource
exists under <code>preburn_address</code>, with a non-zero <code>to_burn</code> field. After the successful execution
of this transaction the <code>total_value</code> field in the
<code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code> resource published under <code>0xA550C18</code> will be
decremented by the value of the <code>to_burn</code> field of the preburn resource
under <code>preburn_address</code> immediately before this transaction, and the
<code>to_burn</code> field of the preburn resource will have a zero value.


<a name="@Events_186"></a>

###### Events

The successful execution of this transaction will emit a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnEvent">Libra::BurnEvent</a></code> on the event handle
held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code> resource's <code>burn_events</code> published under
<code>0xA550C18</code>.


<a name="@Parameters_187"></a>

##### Parameters

| Name              | Type      | Description                                                                                                                  |
| ------            | ------    | -------------                                                                                                                |
| <code>Token</code>           | Type      | The Move type for the <code>Token</code> currency being burned. <code>Token</code> must be an already-registered currency on-chain.                |
| <code>tc_account</code>      | <code>&signer</code> | The signer reference of the sending account of this transaction, must have a burn capability for <code>Token</code> published under it. |
| <code>sliding_nonce</code>   | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                   |
| <code>preburn_address</code> | <code>address</code> | The address where the coins to-be-burned are currently held.                                                                 |


<a name="@Common_Abort_Conditions_188"></a>

##### Common Abort Conditions

| Error Category                | Error Reason                            | Description                                                                                           |
| ----------------              | --------------                          | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EBURN_CAPABILITY">Libra::EBURN_CAPABILITY</a></code>               | The sending <code>account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code> published under it.              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                       | The account at <code>preburn_address</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN_EMPTY">Libra::EPREBURN_EMPTY</a></code>                 | The <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource is empty (has a value of 0).                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                 | The specified <code>Token</code> is not a registered currency on-chain.                                          |


<a name="@Related_Scripts_189"></a>

##### Related Scripts

* <code><a href="overview.md#burn_txn_fees">Script::burn_txn_fees</a></code>
* <code><a href="overview.md#cancel_burn">Script::cancel_burn</a></code>
* <code><a href="overview.md#preburn">Script::preburn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#burn">burn</a>&lt;Token&gt;(account: &signer, sliding_nonce: u64, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#burn">burn</a>&lt;Token&gt;(account: &signer, sliding_nonce: u64, preburn_address: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/Libra.md#0x1_Libra_burn">Libra::burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/Libra.md#0x1_Libra_BurnAbortsIf">Libra::BurnAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/Libra.md#0x1_Libra_BurnEnsures">Libra::BurnEnsures</a>&lt;Token&gt;;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
</code></pre>



</details>

---


<a name="cancel_burn"></a>

#### Script `cancel_burn`



<a name="@Summary_190"></a>

##### Summary

Cancels and returns all coins held in the preburn area under
<code>preburn_address</code> and returns the funds to the <code>preburn_address</code>'s balance.
Can only be successfully sent by an account with Treasury Compliance role.


<a name="@Technical_Description_191"></a>

##### Technical Description

Cancels and returns all coins held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource under the <code>preburn_address</code> and
return the funds to the <code>preburn_address</code> account's <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code>.
The transaction must be sent by an <code>account</code> with a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code>
resource published under it. The account at <code>preburn_address</code> must have a
<code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it, and its value must be nonzero. The transaction removes
the entire balance held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource, and returns it back to the account's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code> under <code>preburn_address</code>. Due to this, the account at
<code>preburn_address</code> must already have a balance in the <code>Token</code> currency published
before this script is called otherwise the transaction will fail.


<a name="@Events_192"></a>

###### Events

The successful execution of this transaction will emit:
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a></code> on the event handle held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code>
resource's <code>burn_events</code> published under <code>0xA550C18</code>.
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on the <code>preburn_address</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> event handle with both the <code>payer</code> and <code>payee</code>
being <code>preburn_address</code>.


<a name="@Parameters_193"></a>

##### Parameters

| Name              | Type      | Description                                                                                                                          |
| ------            | ------    | -------------                                                                                                                        |
| <code>Token</code>           | Type      | The Move type for the <code>Token</code> currenty that burning is being cancelled for. <code>Token</code> must be an already-registered currency on-chain. |
| <code>account</code>         | <code>&signer</code> | The signer reference of the sending account of this transaction, must have a burn capability for <code>Token</code> published under it.         |
| <code>preburn_address</code> | <code>address</code> | The address where the coins to-be-burned are currently held.                                                                         |


<a name="@Common_Abort_Conditions_194"></a>

##### Common Abort Conditions

| Error Category                | Error Reason                                     | Description                                                                                           |
| ----------------              | --------------                                   | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EBURN_CAPABILITY">Libra::EBURN_CAPABILITY</a></code>                        | The sending <code>account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code> published under it.              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                                | The account at <code>preburn_address</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                          | The specified <code>Token</code> is not a registered currency on-chain.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">LibraAccount::ECOIN_DEPOSIT_IS_ZERO</a></code>            | The value held in the preburn resource was zero.                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | The account at <code>preburn_address</code> doesn't have a balance resource for <code>Token</code>.                         |


<a name="@Related_Scripts_195"></a>

##### Related Scripts

* <code><a href="overview.md#burn_txn_fees">Script::burn_txn_fees</a></code>
* <code><a href="overview.md#burn">Script::burn</a></code>
* <code><a href="overview.md#preburn">Script::preburn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_cancel_burn">LibraAccount::cancel_burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_CancelBurnAbortsIf">LibraAccount::CancelBurnAbortsIf</a>&lt;Token&gt;;
<a name="cancel_burn_preburn_value_at_addr$1"></a>
<b>let</b> preburn_value_at_addr = <b>global</b>&lt;<a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;&gt;(preburn_address).to_burn.value;
<a name="cancel_burn_total_preburn_value$2"></a>
<b>let</b> total_preburn_value =
    <b>global</b>&lt;<a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;&gt;(<a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).preburn_value;
<a name="cancel_burn_balance_at_addr$3"></a>
<b>let</b> balance_at_addr = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Token&gt;(preburn_address);
</code></pre>


The value stored at <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a></code> under <code>preburn_address</code> should become zero.


<pre><code><b>ensures</b> preburn_value_at_addr == 0;
</code></pre>


The total value of preburn for <code>Token</code> should decrease by the preburned amount.


<pre><code><b>ensures</b> total_preburn_value == <b>old</b>(total_preburn_value) - <b>old</b>(preburn_value_at_addr);
</code></pre>


The balance of <code>Token</code> at <code>preburn_address</code> should increase by the preburned amount.


<pre><code><b>ensures</b> balance_at_addr == <b>old</b>(balance_at_addr) + <b>old</b>(preburn_value_at_addr);
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>, // TODO: Undocumented error code. Possibly raised in <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_deposit">LibraAccount::deposit</a>, <a href="../../modules/doc/Libra.md#0x1_Libra_deposit">Libra::deposit</a>, and <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_can_receive">AccountLimits::can_receive</a>.
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
</code></pre>



</details>

---


<a name="burn_txn_fees"></a>

#### Script `burn_txn_fees`



<a name="@Summary_196"></a>

##### Summary

Burns the transaction fees collected in the <code>CoinType</code> currency so that the
Libra association may reclaim the backing coins off-chain. May only be sent
by the Treasury Compliance account.


<a name="@Technical_Description_197"></a>

##### Technical Description

Burns the transaction fees collected in <code>CoinType</code> so that the
association may reclaim the backing coins. Once this transaction has executed
successfully all transaction fees that will have been collected in
<code>CoinType</code> since the last time this script was called with that specific
currency. Both <code>balance</code> and <code><a href="overview.md#preburn">preburn</a></code> fields in the
<code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;CoinType&gt;</code> resource published under the <code>0xB1E55ED</code>
account address will have a value of 0 after the successful execution of this script.


<a name="@Events_198"></a>

###### Events

The successful execution of this transaction will emit a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnEvent">Libra::BurnEvent</a></code> on the event handle
held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;CoinType&gt;</code> resource's <code>burn_events</code> published under
<code>0xA550C18</code>.


<a name="@Parameters_199"></a>

##### Parameters

| Name         | Type      | Description                                                                                                                                         |
| ------       | ------    | -------------                                                                                                                                       |
| <code>CoinType</code>   | Type      | The Move type for the <code>CoinType</code> being added to the sending account of the transaction. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code> | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                           |


<a name="@Common_Abort_Conditions_200"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                          | Description                                                 |
| ----------------           | --------------                        | -------------                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code> | The sending account is not the Treasury Compliance account. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">TransactionFee::ETRANSACTION_FEE</a></code>    | <code>CoinType</code> is not an accepted transaction fee currency.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECOIN">Libra::ECOIN</a></code>                        | The collected fees in <code>CoinType</code> are zero.                  |


<a name="@Related_Scripts_201"></a>

##### Related Scripts

* <code><a href="overview.md#burn">Script::burn</a></code>
* <code><a href="overview.md#cancel_burn">Script::cancel_burn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer) {
    <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>&lt;CoinType&gt;(tc_account);
}
</code></pre>



</details>

---


<a name="tiered_mint"></a>

#### Script `tiered_mint`



<a name="@Summary_202"></a>

##### Summary

Mints a specified number of coins in a currency to a Designated Dealer. The sending account
must be the Treasury Compliance account, and coins can only be minted to a Designated Dealer
account.


<a name="@Technical_Description_203"></a>

##### Technical Description

Mints <code>mint_amount</code> of coins in the <code>CoinType</code> currency to Designated Dealer account at
<code>designated_dealer_address</code>. The <code>tier_index</code> parameter specifies which tier should be used to
check verify the off-chain approval policy, and is based in part on the on-chain tier values
for the specific Designated Dealer, and the number of <code>CoinType</code> coins that have been minted to
the dealer over the past 24 hours. Every Designated Dealer has 4 tiers for each currency that
they support. The sending <code>tc_account</code> must be the Treasury Compliance account, and the
receiver an authorized Designated Dealer account.


<a name="@Events_204"></a>

###### Events

Successful execution of the transaction will emit two events:
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_MintEvent">Libra::MintEvent</a></code> with the amount and currency code minted is emitted on the
<code>mint_event_handle</code> in the stored <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;CoinType&gt;</code> resource stored under
<code>0xA550C18</code>; and
* A <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">DesignatedDealer::ReceivedMintEvent</a></code> with the amount, currency code, and Designated
Dealer's address is emitted on the <code>mint_event_handle</code> in the stored <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a></code>
resource published under the <code>designated_dealer_address</code>.


<a name="@Parameters_205"></a>

##### Parameters

| Name                        | Type      | Description                                                                                                |
| ------                      | ------    | -------------                                                                                              |
| <code>CoinType</code>                  | Type      | The Move type for the <code>CoinType</code> being minted. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>                | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.  |
| <code>sliding_nonce</code>             | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                 |
| <code>designated_dealer_address</code> | <code>address</code> | The address of the Designated Dealer account being minted to.                                              |
| <code>mint_amount</code>               | <code>u64</code>     | The number of coins to be minted.                                                                          |
| <code>tier_index</code>                | <code>u64</code>     | The mint tier index to use for the Designated Dealer account.                                              |


<a name="@Common_Abort_Conditions_206"></a>

##### Common Abort Conditions

| Error Category                | Error Reason                                 | Description                                                                                                                  |
| ----------------              | --------------                               | -------------                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>               | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>               | The <code>sliding_nonce</code> is too far in the future.                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>      | The <code>sliding_nonce</code> has been previously recorded.                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>    | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>        | <code>tc_account</code> is not the Treasury Compliance account.                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_MINT_AMOUNT">DesignatedDealer::EINVALID_MINT_AMOUNT</a></code>     | <code>mint_amount</code> is zero.                                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">DesignatedDealer::EDEALER</a></code>                  | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a></code> or <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">DesignatedDealer::TierInfo</a>&lt;CoinType&gt;</code> resource does not exist at <code>designated_dealer_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_INDEX">DesignatedDealer::EINVALID_TIER_INDEX</a></code>      | The <code>tier_index</code> is out of bounds.                                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER">DesignatedDealer::EINVALID_AMOUNT_FOR_TIER</a></code> | <code>mint_amount</code> exceeds the maximum allowed amount for <code>tier_index</code>.                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINT_CAPABILITY">Libra::EMINT_CAPABILITY</a></code>                    | <code>tc_account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;</code> resource published under it.                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINTING_NOT_ALLOWED">Libra::EMINTING_NOT_ALLOWED</a></code>                | Minting is not currently allowed for <code>CoinType</code> coins.                                                                       |


<a name="@Related_Scripts_207"></a>

##### Related Scripts

* <code><a href="overview.md#create_designated_dealer">Script::create_designated_dealer</a></code>
* <code><a href="overview.md#peer_to_peer_with_metadata">Script::peer_to_peer_with_metadata</a></code>
* <code><a href="overview.md#rotate_dual_attestation_info">Script::rotate_dual_attestation_info</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_tiered_mint">LibraAccount::tiered_mint</a>&lt;CoinType&gt;(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: tc_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{account: tc_account, seq_nonce: sliding_nonce};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintAbortsIf">LibraAccount::TieredMintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">LibraAccount::TieredMintEnsures</a>&lt;CoinType&gt;;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>, // TODO: Undocumented error code. Possibly raised in <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_deposit">LibraAccount::deposit</a>, <a href="../../modules/doc/Libra.md#0x1_Libra_deposit">Libra::deposit</a>, and <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_can_receive">AccountLimits::can_receive</a>.
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>



</details>

---


<a name="freeze_account"></a>

#### Script `freeze_account`



<a name="@Summary_208"></a>

##### Summary

Freezes the account at <code>address</code>. The sending account of this transaction
must be the Treasury Compliance account. The account being frozen cannot be
the Libra Root or Treasury Compliance account. After the successful
execution of this transaction no transactions may be sent from the frozen
account, and the frozen account may not send or receive coins.


<a name="@Technical_Description_209"></a>

##### Technical Description

Sets the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a></code> to <code><b>true</b></code> and emits a
<code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a></code>. The transaction sender must be the
Treasury Compliance account, but the account at <code>to_freeze_account</code> must
not be either <code>0xA550C18</code> (the Libra Root address), or <code>0xB1E55ED</code> (the
Treasury Compliance address). Note that this is a per-account property
e.g., freezing a Parent VASP will not effect the status any of its child
accounts and vice versa.



<a name="@Events_210"></a>

###### Events

Successful execution of this transaction will emit a <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a></code> on
the <code>freeze_event_handle</code> held in the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">AccountFreezing::FreezeEventsHolder</a></code> resource published
under <code>0xA550C18</code> with the <code>frozen_address</code> being the <code>to_freeze_account</code>.


<a name="@Parameters_211"></a>

##### Parameters

| Name                | Type      | Description                                                                                               |
| ------              | ------    | -------------                                                                                             |
| <code>tc_account</code>        | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>     | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>to_freeze_account</code> | <code>address</code> | The account address to be frozen.                                                                         |


<a name="@Common_Abort_Conditions_212"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                 | Description                                                                                |
| ----------------           | --------------                               | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>               | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>               | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>      | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>        | The sending account is not the Treasury Compliance account.                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_TC">AccountFreezing::ECANNOT_FREEZE_TC</a></code>         | <code>to_freeze_account</code> was the Treasury Compliance account (<code>0xB1E55ED</code>).                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT">AccountFreezing::ECANNOT_FREEZE_LIBRA_ROOT</a></code> | <code>to_freeze_account</code> was the Libra Root account (<code>0xA550C18</code>).                              |


<a name="@Related_Scripts_213"></a>

##### Related Scripts

* <code>Scripts::unfreeze_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#freeze_account">freeze_account</a>(tc_account: &signer, sliding_nonce: u64, to_freeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#freeze_account">freeze_account</a>(tc_account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_freeze_account">AccountFreezing::freeze_account</a>(tc_account, to_freeze_account);
}
</code></pre>



</details>

---


<a name="unfreeze_account"></a>

#### Script `unfreeze_account`



<a name="@Summary_214"></a>

##### Summary

Unfreezes the account at <code>address</code>. The sending account of this transaction must be the
Treasury Compliance account. After the successful execution of this transaction transactions
may be sent from the previously frozen account, and coins may be sent and received.


<a name="@Technical_Description_215"></a>

##### Technical Description

Sets the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a></code> to <code><b>false</b></code> and emits a
<code>AccountFreezing::UnFreezeAccountEvent</code>. The transaction sender must be the Treasury Compliance
account. Note that this is a per-account property so unfreezing a Parent VASP will not effect
the status any of its child accounts and vice versa.


<a name="@Events_216"></a>

###### Events

Successful execution of this script will emit a <code>AccountFreezing::UnFreezeAccountEvent</code> with
the <code>unfrozen_address</code> set the <code>to_unfreeze_account</code>'s address.


<a name="@Parameters_217"></a>

##### Parameters

| Name                  | Type      | Description                                                                                               |
| ------                | ------    | -------------                                                                                             |
| <code>tc_account</code>          | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>       | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>to_unfreeze_account</code> | <code>address</code> | The account address to be frozen.                                                                         |


<a name="@Common_Abort_Conditions_218"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | The sending account is not the Treasury Compliance account.                                |


<a name="@Related_Scripts_219"></a>

##### Related Scripts

* <code>Scripts::freeze_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_unfreeze_account">AccountFreezing::unfreeze_account</a>(account, to_unfreeze_account);
}
</code></pre>



</details>

---


<a name="update_dual_attestation_limit"></a>

#### Script `update_dual_attestation_limit`



<a name="@Summary_220"></a>

##### Summary

Update the dual attestation limit on-chain. Defined in terms of micro-LBR.  The transaction can
only be sent by the Treasury Compliance account.  After this transaction all inter-VASP
payments over this limit must be checked for dual attestation.


<a name="@Technical_Description_221"></a>

##### Technical Description

Updates the <code>micro_lbr_limit</code> field of the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Limit">DualAttestation::Limit</a></code> resource published under
<code>0xA550C18</code>. The amount is set in micro-LBR.


<a name="@Parameters_222"></a>

##### Parameters

| Name                  | Type      | Description                                                                                               |
| ------                | ------    | -------------                                                                                             |
| <code>tc_account</code>          | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>       | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>new_micro_lbr_limit</code> | <code>u64</code>     | The new dual attestation limit to be used on-chain.                                                       |


<a name="@Common_Abort_Conditions_223"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | <code>tc_account</code> is not the Treasury Compliance account.                                       |


<a name="@Related_Scripts_224"></a>

##### Related Scripts

* <code>Scripts::update_exchange_rate</code>
* <code>Scripts::update_minting_ability</code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#update_dual_attestation_limit">update_dual_attestation_limit</a>(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#update_dual_attestation_limit">update_dual_attestation_limit</a>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_micro_lbr_limit: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_set_microlibra_limit">DualAttestation::set_microlibra_limit</a>(tc_account, new_micro_lbr_limit);
}
</code></pre>



</details>

---


<a name="update_exchange_rate"></a>

#### Script `update_exchange_rate`



<a name="@Summary_225"></a>

##### Summary

Update the rough on-chain exchange rate between a specified currency and LBR (as a conversion
to micro-LBR). The transaction can only be sent by the Treasury Compliance account. After this
transaction the updated exchange rate will be used for normalization of gas prices, and for
dual attestation checking.


<a name="@Technical_Description_226"></a>

##### Technical Description

Updates the on-chain exchange rate from the given <code>Currency</code> to micro-LBR.  The exchange rate
is given by <code>new_exchange_rate_numerator/new_exchange_rate_denominator</code>.


<a name="@Parameters_227"></a>

##### Parameters

| Name                            | Type      | Description                                                                                                                        |
| ------                          | ------    | -------------                                                                                                                      |
| <code>Currency</code>                      | Type      | The Move type for the <code>Currency</code> whose exchange rate is being updated. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>                    | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                          |
| <code>sliding_nonce</code>                 | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for the transaction.                                                          |
| <code>new_exchange_rate_numerator</code>   | <code>u64</code>     | The numerator for the new to micro-LBR exchange rate for <code>Currency</code>.                                                               |
| <code>new_exchange_rate_denominator</code> | <code>u64</code>     | The denominator for the new to micro-LBR exchange rate for <code>Currency</code>.                                                             |


<a name="@Common_Abort_Conditions_228"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | <code>tc_account</code> is not the Treasury Compliance account.                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">FixedPoint32::EDENOMINATOR</a></code>            | <code>new_exchange_rate_denominator</code> is zero.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">FixedPoint32::ERATIO_OUT_OF_RANGE</a></code>     | The quotient is unrepresentable as a <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>.                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">FixedPoint32::ERATIO_OUT_OF_RANGE</a></code>     | The quotient is unrepresentable as a <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>.                                       |


<a name="@Related_Scripts_229"></a>

##### Related Scripts

* <code>Scripts::update_dual_attestation_limit</code>
* <code>Scripts::update_minting_ability</code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, new_exchange_rate_numerator: u64, new_exchange_rate_denominator: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> rate = <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        new_exchange_rate_numerator,
        new_exchange_rate_denominator,
    );
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_lbr_exchange_rate">Libra::update_lbr_exchange_rate</a>&lt;Currency&gt;(tc_account, rate);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TransactionChecks">LibraAccount::TransactionChecks</a>{sender: tc_account};
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ account: tc_account, seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_CreateFromRationalAbortsIf">FixedPoint32::CreateFromRationalAbortsIf</a>{
    numerator: new_exchange_rate_numerator,
    denominator: new_exchange_rate_denominator
};
<a name="update_exchange_rate_rate$1"></a>
<b>let</b> rate = <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">FixedPoint32::spec_create_from_rational</a>(
    new_exchange_rate_numerator,
    new_exchange_rate_denominator
);
<b>include</b> <a href="../../modules/doc/Libra.md#0x1_Libra_UpdateLBRExchangeRateAbortsIf">Libra::UpdateLBRExchangeRateAbortsIf</a>&lt;Currency&gt;;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>, // TODO: Undocumented error code. Can be raised in <a href="../../modules/doc/Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>.
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

---


<a name="update_minting_ability"></a>

#### Script `update_minting_ability`



<a name="@Summary_230"></a>

##### Summary

Script to allow or disallow minting of new coins in a specified currency.  This transaction can
only be sent by the Treasury Compliance account.  Turning minting off for a currency will have
no effect on coins already in circulation, and coins may still be removed from the system.


<a name="@Technical_Description_231"></a>

##### Technical Description

This transaction sets the <code>can_mint</code> field of the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Currency&gt;</code> resource
published under <code>0xA550C18</code> to the value of <code>allow_minting</code>. Minting of coins if allowed if
this field is set to <code><b>true</b></code> and minting of new coins in <code>Currency</code> is disallowed otherwise.
This transaction needs to be sent by the Treasury Compliance account.


<a name="@Parameters_232"></a>

##### Parameters

| Name            | Type      | Description                                                                                                                          |
| ------          | ------    | -------------                                                                                                                        |
| <code>Currency</code>      | Type      | The Move type for the <code>Currency</code> whose minting ability is being updated. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>account</code>       | <code>&signer</code> | Signer reference of the sending account. Must be the Libra Root account.                                                             |
| <code>allow_minting</code> | <code>bool</code>    | Whether to allow minting of new coins in <code>Currency</code>.                                                                                 |


<a name="@Common_Abort_Conditions_233"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                          | Description                                          |
| ----------------           | --------------                        | -------------                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code> | <code>tc_account</code> is not the Treasury Compliance account. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>               | <code>Currency</code> is not a registered currency on-chain.    |


<a name="@Related_Scripts_234"></a>

##### Related Scripts

* <code>Scripts::update_dual_attestation_limit</code>
* <code>Scripts::update_exchange_rate</code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(tc_account: &signer, allow_minting: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(
    tc_account: &signer,
    allow_minting: bool
) {
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_minting_ability">Libra::update_minting_ability</a>&lt;Currency&gt;(tc_account, allow_minting);
}
</code></pre>



</details>


---

<a name="@System_Administration_235"></a>

### System Administration



<a name="update_libra_version"></a>

#### Script `update_libra_version`



<a name="@Summary_236"></a>

##### Summary

Updates the Libra major version that is stored on-chain and is used by the VM.  This
transaction can only be sent from the Libra Root account.


<a name="@Technical_Description_237"></a>

##### Technical Description

Updates the <code><a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion">LibraVersion</a></code> on-chain config and emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> to trigger
a reconfiguration of the system. The <code>major</code> version that is passed in must be strictly greater
than the current major version held on-chain. The VM reads this information and can use it to
preserve backwards compatibility with previous major versions of the VM.


<a name="@Parameters_238"></a>

##### Parameters

| Name            | Type      | Description                                                                |
| ------          | ------    | -------------                                                              |
| <code>account</code>       | <code>&signer</code> | Signer reference of the sending account. Must be the Libra Root account.   |
| <code>sliding_nonce</code> | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction. |
| <code>major</code>         | <code>u64</code>     | The <code>major</code> version of the VM to be used from this transaction on.         |


<a name="@Common_Abort_Conditions_239"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                  | Description                                                                                |
| ----------------           | --------------                                | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>       | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>                  | <code>account</code> is not the Libra Root account.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER">LibraVersion::EINVALID_MAJOR_VERSION_NUMBER</a></code> | <code>major</code> is less-than or equal to the current major version stored on-chain.                |


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion_set">LibraVersion::set</a>(account, major)
}
</code></pre>



</details>

---


<a name="add_to_script_allow_list"></a>

#### Script `add_to_script_allow_list`



<a name="@Summary_240"></a>

##### Summary

Adds a script hash to the transaction allowlist. This transaction
can only be sent by the Libra Root account. Scripts with this hash can be
sent afterward the successful execution of this script.


<a name="@Technical_Description_241"></a>

##### Technical Description


The sending account (<code>lr_account</code>) must be the Libra Root account. The script allow
list must not already hold the script <code>hash</code> being added. The <code>sliding_nonce</code> must be
a valid nonce for the Libra Root account. After this transaction has executed
successfully a reconfiguration will be initiated, and the on-chain config
<code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption">LibraTransactionPublishingOption::LibraTransactionPublishingOption</a></code>'s
<code>script_allow_list</code> field will contain the new script <code>hash</code> and transactions
with this <code>hash</code> can be successfully sent to the network.


<a name="@Parameters_242"></a>

##### Parameters

| Name            | Type         | Description                                                                                     |
| ------          | ------       | -------------                                                                                   |
| <code>lr_account</code>    | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
| <code>hash</code>          | <code>vector&lt;u8&gt;</code> | The hash of the script to be added to the script allowlist.                                     |
| <code>sliding_nonce</code> | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                      |


<a name="@Common_Abort_Conditions_243"></a>

##### Common Abort Conditions

| Error Category             | Error Reason                                                           | Description                                                                                |
| ----------------           | --------------                                                         | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>                                           | The sending account is not the Libra Root account.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                                         | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                                         | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>                                | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">LibraTransactionPublishingOption::EINVALID_SCRIPT_HASH</a></code>               | The script <code>hash</code> is an invalid length.                                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">LibraTransactionPublishingOption::EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a></code> | The on-chain allowlist already contains the script <code>hash</code>.                                 |


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64,) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">LibraTransactionPublishingOption::add_to_script_allow_list</a>(lr_account, hash)
}
</code></pre>



</details>



<a name="@Index_244"></a>

### Index


-  [0x1::AccountFreezing](../../modules/doc/AccountFreezing.md#0x1_AccountFreezing)
-  [0x1::AccountLimits](../../modules/doc/AccountLimits.md#0x1_AccountLimits)
-  [0x1::Authenticator](../../modules/doc/Authenticator.md#0x1_Authenticator)
-  [0x1::ChainId](../../modules/doc/ChainId.md#0x1_ChainId)
-  [0x1::Coin1](../../modules/doc/Coin1.md#0x1_Coin1)
-  [0x1::Coin2](../../modules/doc/Coin2.md#0x1_Coin2)
-  [0x1::CoreAddresses](../../modules/doc/CoreAddresses.md#0x1_CoreAddresses)
-  [0x1::DesignatedDealer](../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer)
-  [0x1::DualAttestation](../../modules/doc/DualAttestation.md#0x1_DualAttestation)
-  [0x1::Errors](../../modules/doc/Errors.md#0x1_Errors)
-  [0x1::Event](../../modules/doc/Event.md#0x1_Event)
-  [0x1::FixedPoint32](../../modules/doc/FixedPoint32.md#0x1_FixedPoint32)
-  [0x1::Hash](../../modules/doc/Hash.md#0x1_Hash)
-  [0x1::LBR](../../modules/doc/LBR.md#0x1_LBR)
-  [0x1::LCS](../../modules/doc/LCS.md#0x1_LCS)
-  [0x1::Libra](../../modules/doc/Libra.md#0x1_Libra)
-  [0x1::LibraAccount](../../modules/doc/LibraAccount.md#0x1_LibraAccount)
-  [0x1::LibraConfig](../../modules/doc/LibraConfig.md#0x1_LibraConfig)
-  [0x1::LibraSystem](../../modules/doc/LibraSystem.md#0x1_LibraSystem)
-  [0x1::LibraTimestamp](../../modules/doc/LibraTimestamp.md#0x1_LibraTimestamp)
-  [0x1::LibraTransactionPublishingOption](../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption)
-  [0x1::LibraVersion](../../modules/doc/LibraVersion.md#0x1_LibraVersion)
-  [0x1::Option](../../modules/doc/Option.md#0x1_Option)
-  [0x1::RecoveryAddress](../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress)
-  [0x1::RegisteredCurrencies](../../modules/doc/RegisteredCurrencies.md#0x1_RegisteredCurrencies)
-  [0x1::Roles](../../modules/doc/Roles.md#0x1_Roles)
-  [0x1::SharedEd25519PublicKey](../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey)
-  [0x1::Signature](../../modules/doc/Signature.md#0x1_Signature)
-  [0x1::Signer](../../modules/doc/Signer.md#0x1_Signer)
-  [0x1::SlidingNonce](../../modules/doc/SlidingNonce.md#0x1_SlidingNonce)
-  [0x1::TransactionFee](../../modules/doc/TransactionFee.md#0x1_TransactionFee)
-  [0x1::VASP](../../modules/doc/VASP.md#0x1_VASP)
-  [0x1::ValidatorConfig](../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig)
-  [0x1::ValidatorOperatorConfig](../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig)
-  [0x1::Vector](../../modules/doc/Vector.md#0x1_Vector)
-  [add_currency_to_account](overview.md#add_currency_to_account)
-  [add_recovery_rotation_capability](overview.md#add_recovery_rotation_capability)
-  [add_to_script_allow_list](overview.md#add_to_script_allow_list)
-  [add_validator_and_reconfigure](overview.md#add_validator_and_reconfigure)
-  [burn](overview.md#burn)
-  [burn_txn_fees](overview.md#burn_txn_fees)
-  [cancel_burn](overview.md#cancel_burn)
-  [create_child_vasp_account](overview.md#create_child_vasp_account)
-  [create_designated_dealer](overview.md#create_designated_dealer)
-  [create_parent_vasp_account](overview.md#create_parent_vasp_account)
-  [create_recovery_address](overview.md#create_recovery_address)
-  [create_validator_account](overview.md#create_validator_account)
-  [create_validator_operator_account](overview.md#create_validator_operator_account)
-  [freeze_account](overview.md#freeze_account)
-  [mint_lbr](overview.md#mint_lbr)
-  [peer_to_peer_with_metadata](overview.md#peer_to_peer_with_metadata)
-  [preburn](overview.md#preburn)
-  [publish_shared_ed25519_public_key](overview.md#publish_shared_ed25519_public_key)
-  [register_validator_config](overview.md#register_validator_config)
-  [remove_validator_and_reconfigure](overview.md#remove_validator_and_reconfigure)
-  [rotate_authentication_key](overview.md#rotate_authentication_key)
-  [rotate_authentication_key_with_nonce](overview.md#rotate_authentication_key_with_nonce)
-  [rotate_authentication_key_with_nonce_admin](overview.md#rotate_authentication_key_with_nonce_admin)
-  [rotate_authentication_key_with_recovery_address](overview.md#rotate_authentication_key_with_recovery_address)
-  [rotate_dual_attestation_info](overview.md#rotate_dual_attestation_info)
-  [rotate_shared_ed25519_public_key](overview.md#rotate_shared_ed25519_public_key)
-  [set_validator_config_and_reconfigure](overview.md#set_validator_config_and_reconfigure)
-  [set_validator_operator](overview.md#set_validator_operator)
-  [set_validator_operator_with_nonce_admin](overview.md#set_validator_operator_with_nonce_admin)
-  [tiered_mint](overview.md#tiered_mint)
-  [unfreeze_account](overview.md#unfreeze_account)
-  [unmint_lbr](overview.md#unmint_lbr)
-  [update_dual_attestation_limit](overview.md#update_dual_attestation_limit)
-  [update_exchange_rate](overview.md#update_exchange_rate)
-  [update_libra_version](overview.md#update_libra_version)
-  [update_minting_ability](overview.md#update_minting_ability)
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
