
<a name="0x1_ValidatorAdministrationScripts"></a>

# Module `0x1::ValidatorAdministrationScripts`



-  [Function `add_validator_and_reconfigure`](#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure)
    -  [Summary](#@Summary_0)
    -  [Technical Description](#@Technical_Description_1)
    -  [Parameters](#@Parameters_2)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_3)
    -  [Related Scripts](#@Related_Scripts_4)
-  [Function `register_validator_config`](#0x1_ValidatorAdministrationScripts_register_validator_config)
    -  [Summary](#@Summary_5)
    -  [Technical Description](#@Technical_Description_6)
    -  [Parameters](#@Parameters_7)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_8)
    -  [Related Scripts](#@Related_Scripts_9)
-  [Function `remove_validator_and_reconfigure`](#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure)
    -  [Summary](#@Summary_10)
    -  [Technical Description](#@Technical_Description_11)
    -  [Parameters](#@Parameters_12)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_13)
    -  [Related Scripts](#@Related_Scripts_14)
-  [Function `set_validator_config_and_reconfigure`](#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure)
    -  [Summary](#@Summary_15)
    -  [Technical Description](#@Technical_Description_16)
    -  [Parameters](#@Parameters_17)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_18)
    -  [Related Scripts](#@Related_Scripts_19)
-  [Function `set_validator_operator`](#0x1_ValidatorAdministrationScripts_set_validator_operator)
    -  [Summary](#@Summary_20)
    -  [Technical Description](#@Technical_Description_21)
    -  [Parameters](#@Parameters_22)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_23)
    -  [Related Scripts](#@Related_Scripts_24)
-  [Function `set_validator_operator_with_nonce_admin`](#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin)
    -  [Summary](#@Summary_25)
    -  [Technical Description](#@Technical_Description_26)
    -  [Parameters](#@Parameters_27)
    -  [Common Abort Conditions](#@Common_Abort_Conditions_28)
    -  [Related Scripts](#@Related_Scripts_29)


<pre><code><b>use</b> <a href="DiemSystem.md#0x1_DiemSystem">0x1::DiemSystem</a>;
<b>use</b> <a href="SlidingNonce.md#0x1_SlidingNonce">0x1::SlidingNonce</a>;
<b>use</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">0x1::ValidatorConfig</a>;
<b>use</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">0x1::ValidatorOperatorConfig</a>;
</code></pre>



<a name="0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure"></a>

## Function `add_validator_and_reconfigure`


<a name="@Summary_0"></a>

### Summary

Adds a validator account to the validator set, and triggers a
reconfiguration of the system to admit the account to the validator set for the system. This
transaction can only be successfully called by the Diem Root account.


<a name="@Technical_Description_1"></a>

### Technical Description

This script adds the account at <code>validator_address</code> to the validator set.
This transaction emits a <code><a href="DiemConfig.md#0x1_DiemConfig_NewEpochEvent">DiemConfig::NewEpochEvent</a></code> event and triggers a
reconfiguration. Once the reconfiguration triggered by this script's
execution has been performed, the account at the <code>validator_address</code> is
considered to be a validator in the network.

This transaction script will fail if the <code>validator_address</code> address is already in the validator set
or does not have a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource already published under it.


<a name="@Parameters_2"></a>

### Parameters

| Name                | Type         | Description                                                                                                                        |
| ------              | ------       | -------------                                                                                                                      |
| <code>dr_account</code>        | <code>signer</code>     | The signer of the sending account of this transaction. Must be the Diem Root signer.                                               |
| <code>sliding_nonce</code>     | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                         |
| <code>validator_name</code>    | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator. Must match the human name in the <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> for the validator. |
| <code>validator_address</code> | <code>address</code>    | The validator account address to be added to the validator set.                                                                    |


<a name="@Common_Abort_Conditions_3"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                 | Description                                                                                                                               |
| ----------------           | --------------                               | -------------                                                                                                                             |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>               | A <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> resource is not published under <code>dr_account</code>.                                                                            |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>               | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.                                                |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>               | The <code>sliding_nonce</code> is too far in the future.                                                                                             |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>      | The <code>sliding_nonce</code> has been previously recorded.                                                                                         |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="CoreAddresses.md#0x1_CoreAddresses_EDIEM_ROOT">CoreAddresses::EDIEM_ROOT</a></code>                  | The sending account is not the Diem Root account.                                                                                         |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="Roles.md#0x1_Roles_EDIEM_ROOT">Roles::EDIEM_ROOT</a></code>                          | The sending account is not the Diem Root account.                                                                                         |
| 0                          | 0                                            | The provided <code>validator_name</code> does not match the already-recorded human name for the validator.                                           |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="DiemSystem.md#0x1_DiemSystem_EINVALID_PROSPECTIVE_VALIDATOR">DiemSystem::EINVALID_PROSPECTIVE_VALIDATOR</a></code> | The validator to be added does not have a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it, or its <code>config</code> field is empty. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="DiemSystem.md#0x1_DiemSystem_EALREADY_A_VALIDATOR">DiemSystem::EALREADY_A_VALIDATOR</a></code>           | The <code>validator_address</code> account is already a registered validator.                                                                        |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="DiemConfig.md#0x1_DiemConfig_EINVALID_BLOCK_TIME">DiemConfig::EINVALID_BLOCK_TIME</a></code>            | An invalid time value was encountered in reconfiguration. Unlikely to occur.                                                              |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="DiemSystem.md#0x1_DiemSystem_EMAX_VALIDATORS">DiemSystem::EMAX_VALIDATORS</a></code>                | The validator set is already at its maximum size. The validator could not be added.                                                       |


<a name="@Related_Scripts_4"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">ValidatorAdministrationScripts::register_validator_config</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">ValidatorAdministrationScripts::remove_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">ValidatorAdministrationScripts::set_validator_operator</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">ValidatorAdministrationScripts::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">add_validator_and_reconfigure</a>(dr_account: signer, sliding_nonce: u64, validator_name: vector&lt;u8&gt;, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">add_validator_and_reconfigure</a>(
    dr_account: signer,
    sliding_nonce: u64,
    validator_name: vector&lt;u8&gt;,
    validator_address: address
) {
    <a href="SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(&dr_account, sliding_nonce);
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) == validator_name, 0);
    <a href="DiemSystem.md#0x1_DiemSystem_add_validator">DiemSystem::add_validator</a>(&dr_account, validator_address);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: dr_account};
<b>include</b> <a href="SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{seq_nonce: sliding_nonce, account: dr_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">ValidatorConfig::AbortsIfNoValidatorConfig</a>{addr: validator_address};
<b>aborts_if</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) != validator_name <b>with</b> 0;
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_AddValidatorAbortsIf">DiemSystem::AddValidatorAbortsIf</a>{validator_addr: validator_address};
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_AddValidatorEnsures">DiemSystem::AddValidatorEnsures</a>{validator_addr: validator_address};
</code></pre>


Reports INVALID_STATE because of is_operating() and !exists<DiemSystem::CapabilityHolder>.
is_operating() is always true during transactions, and CapabilityHolder is published
during initialization (Genesis).
Reports REQUIRES_ROLE if dr_account is not Diem root, but that can't happen
in practice because it aborts with NOT_PUBLISHED or REQUIRES_ADDRESS, first.


<pre><code><b>aborts_with</b> [check]
    0, // Odd error code in <b>assert</b> on second statement in add_validator_and_reconfigure
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_ReconfigureEmits">DiemConfig::ReconfigureEmits</a>;
</code></pre>


**Access Control:**
Only the Diem Root account can add Validators [[H14]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
</code></pre>



</details>

<a name="0x1_ValidatorAdministrationScripts_register_validator_config"></a>

## Function `register_validator_config`


<a name="@Summary_5"></a>

### Summary

Updates a validator's configuration. This does not reconfigure the system and will not update
the configuration in the validator set that is seen by other validators in the network. Can
only be successfully sent by a Validator Operator account that is already registered with a
validator.


<a name="@Technical_Description_6"></a>

### Technical Description

This updates the fields with corresponding names held in the <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It does not emit a <code><a href="DiemConfig.md#0x1_DiemConfig_NewEpochEvent">DiemConfig::NewEpochEvent</a></code>
so the copy of this config held in the validator set will not be updated, and the changes are
only "locally" under the <code>validator_account</code> account address.


<a name="@Parameters_7"></a>

### Parameters

| Name                          | Type         | Description                                                                                                        |
| ------                        | ------       | -------------                                                                                                      |
| <code>validator_operator_account</code>  | <code>signer</code>     | Signer of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                          |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                               |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.             |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.              |


<a name="@Common_Abort_Conditions_8"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |


<a name="@Related_Scripts_9"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">ValidatorAdministrationScripts::add_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">ValidatorAdministrationScripts::remove_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">ValidatorAdministrationScripts::set_validator_operator</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">ValidatorAdministrationScripts::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">register_validator_config</a>(validator_operator_account: signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">register_validator_config</a>(
    validator_operator_account: signer,
    // TODO Rename <b>to</b> validator_addr, since it is an address.
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) {
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        &validator_operator_account,
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


<pre><code><b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: validator_operator_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetConfigAbortsIf">ValidatorConfig::SetConfigAbortsIf</a> {validator_addr: validator_account};
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_account);
<b>aborts_with</b> [check]
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>


**Access Control:**
Only the Validator Operator account which has been registered with the validator can
update the validator's configuration [[H15]][PERMISSION].


<pre><code><b>aborts_if</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account) !=
            <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">ValidatorConfig::get_operator</a>(validator_account)
                <b>with</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

<a name="0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure"></a>

## Function `remove_validator_and_reconfigure`


<a name="@Summary_10"></a>

### Summary

This script removes a validator account from the validator set, and triggers a reconfiguration
of the system to remove the validator from the system. This transaction can only be
successfully called by the Diem Root account.


<a name="@Technical_Description_11"></a>

### Technical Description

This script removes the account at <code>validator_address</code> from the validator set. This transaction
emits a <code><a href="DiemConfig.md#0x1_DiemConfig_NewEpochEvent">DiemConfig::NewEpochEvent</a></code> event. Once the reconfiguration triggered by this event
has been performed, the account at <code>validator_address</code> is no longer considered to be a
validator in the network. This transaction will fail if the validator at <code>validator_address</code>
is not in the validator set.


<a name="@Parameters_12"></a>

### Parameters

| Name                | Type         | Description                                                                                                                        |
| ------              | ------       | -------------                                                                                                                      |
| <code>dr_account</code>        | <code>signer</code>     | The signer of the sending account of this transaction. Must be the Diem Root signer.                                               |
| <code>sliding_nonce</code>     | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                         |
| <code>validator_name</code>    | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator. Must match the human name in the <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> for the validator. |
| <code>validator_address</code> | <code>address</code>    | The validator account address to be removed from the validator set.                                                                |


<a name="@Common_Abort_Conditions_13"></a>

### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                     |
| ----------------           | --------------                          | -------------                                                                                   |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>          | A <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> resource is not published under <code>dr_account</code>.                                  |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.      |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                                   |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                               |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>          | The sending account is not the Diem Root account or Treasury Compliance account                |
| 0                          | 0                                       | The provided <code>validator_name</code> does not match the already-recorded human name for the validator. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="DiemSystem.md#0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR">DiemSystem::ENOT_AN_ACTIVE_VALIDATOR</a></code> | The validator to be removed is not in the validator set.                                        |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="CoreAddresses.md#0x1_CoreAddresses_EDIEM_ROOT">CoreAddresses::EDIEM_ROOT</a></code>            | The sending account is not the Diem Root account.                                              |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="Roles.md#0x1_Roles_EDIEM_ROOT">Roles::EDIEM_ROOT</a></code>                    | The sending account is not the Diem Root account.                                              |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="DiemConfig.md#0x1_DiemConfig_EINVALID_BLOCK_TIME">DiemConfig::EINVALID_BLOCK_TIME</a></code>      | An invalid time value was encountered in reconfiguration. Unlikely to occur.                    |


<a name="@Related_Scripts_14"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">ValidatorAdministrationScripts::register_validator_config</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">ValidatorAdministrationScripts::add_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">ValidatorAdministrationScripts::set_validator_operator</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">ValidatorAdministrationScripts::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">remove_validator_and_reconfigure</a>(dr_account: signer, sliding_nonce: u64, validator_name: vector&lt;u8&gt;, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">remove_validator_and_reconfigure</a>(
    dr_account: signer,
    sliding_nonce: u64,
    validator_name: vector&lt;u8&gt;,
    validator_address: address
) {
    <a href="SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(&dr_account, sliding_nonce);
    // TODO: Use an error code from <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">Errors</a>.<b>move</b>
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) == validator_name, 0);
    <a href="DiemSystem.md#0x1_DiemSystem_remove_validator">DiemSystem::remove_validator</a>(&dr_account, validator_address);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: dr_account};
<b>include</b> <a href="SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{seq_nonce: sliding_nonce, account: dr_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">ValidatorConfig::AbortsIfNoValidatorConfig</a>{addr: validator_address};
<b>aborts_if</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) != validator_name <b>with</b> 0;
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_RemoveValidatorAbortsIf">DiemSystem::RemoveValidatorAbortsIf</a>{validator_addr: validator_address};
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_RemoveValidatorEnsures">DiemSystem::RemoveValidatorEnsures</a>{validator_addr: validator_address};
</code></pre>


Reports INVALID_STATE because of is_operating() and !exists<DiemSystem::CapabilityHolder>.
is_operating() is always true during transactions, and CapabilityHolder is published
during initialization (Genesis).
Reports REQUIRES_ROLE if dr_account is not Diem root, but that can't happen
in practice because it aborts with NOT_PUBLISHED or REQUIRES_ADDRESS, first.


<pre><code><b>aborts_with</b> [check]
    0, // Odd error code in <b>assert</b> on second statement in add_validator_and_reconfigure
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_ReconfigureEmits">DiemConfig::ReconfigureEmits</a>;
</code></pre>


**Access Control:**
Only the Diem Root account can remove Validators [[H14]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
</code></pre>



</details>

<a name="0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure"></a>

## Function `set_validator_config_and_reconfigure`


<a name="@Summary_15"></a>

### Summary

Updates a validator's configuration, and triggers a reconfiguration of the system to update the
validator set with this new validator configuration.  Can only be successfully sent by a
Validator Operator account that is already registered with a validator.


<a name="@Technical_Description_16"></a>

### Technical Description

This updates the fields with corresponding names held in the <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It then emits a <code><a href="DiemConfig.md#0x1_DiemConfig_NewEpochEvent">DiemConfig::NewEpochEvent</a></code> to
trigger a reconfiguration of the system.  This reconfiguration will update the validator set
on-chain with the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.


<a name="@Parameters_17"></a>

### Parameters

| Name                          | Type         | Description                                                                                                        |
| ------                        | ------       | -------------                                                                                                      |
| <code>validator_operator_account</code>  | <code>signer</code>     | Signer of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                          |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                               |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.             |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.              |


<a name="@Common_Abort_Conditions_18"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="Roles.md#0x1_Roles_EVALIDATOR_OPERATOR">Roles::EVALIDATOR_OPERATOR</a></code>                   | <code>validator_operator_account</code> does not have a Validator Operator role.                                 |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="DiemConfig.md#0x1_DiemConfig_EINVALID_BLOCK_TIME">DiemConfig::EINVALID_BLOCK_TIME</a></code>             | An invalid time value was encountered in reconfiguration. Unlikely to occur.                          |


<a name="@Related_Scripts_19"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">ValidatorAdministrationScripts::add_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">ValidatorAdministrationScripts::remove_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">ValidatorAdministrationScripts::set_validator_operator</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">ValidatorAdministrationScripts::register_validator_config</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(validator_operator_account: signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(
    validator_operator_account: signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) {
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        &validator_operator_account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses
    );
    <a href="DiemSystem.md#0x1_DiemSystem_update_config_and_reconfigure">DiemSystem::update_config_and_reconfigure</a>(&validator_operator_account, validator_account);
 }
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: validator_operator_account};
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_UpdateConfigAndReconfigureEnsures">DiemSystem::UpdateConfigAndReconfigureEnsures</a>{validator_addr: validator_account};
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_account);
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">ValidatorConfig::spec_get_config</a>(validator_account)
    == <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a> {
                consensus_pubkey,
                validator_network_addresses,
                fullnode_network_addresses,
};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetConfigAbortsIf">ValidatorConfig::SetConfigAbortsIf</a>{validator_addr: validator_account};
<b>include</b> <a href="DiemSystem.md#0x1_DiemSystem_UpdateConfigAndReconfigureAbortsIf">DiemSystem::UpdateConfigAndReconfigureAbortsIf</a>{validator_addr: validator_account};
<b>let</b> is_validator_info_updated =
    (<b>exists</b> v_info in <a href="DiemSystem.md#0x1_DiemSystem_spec_get_validators">DiemSystem::spec_get_validators</a>():
        v_info.addr == validator_account
        && v_info.config != <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a> {
                consensus_pubkey,
                validator_network_addresses,
                fullnode_network_addresses,
           });
<b>include</b> is_validator_info_updated ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_ReconfigureAbortsIf">DiemConfig::ReconfigureAbortsIf</a>;
</code></pre>


This reports a possible INVALID_STATE abort, which comes from an assert in DiemConfig::reconfigure_
that config.last_reconfiguration_time is not in the future. This is a system error that a user
for which there is no useful recovery except to resubmit the transaction.


<pre><code><b>aborts_with</b> [check]
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
<b>include</b> is_validator_info_updated ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_ReconfigureEmits">DiemConfig::ReconfigureEmits</a>;
</code></pre>


**Access Control:**
Only the Validator Operator account which has been registered with the validator can
update the validator's configuration [[H15]][PERMISSION].


<pre><code><b>aborts_if</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account) !=
            <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">ValidatorConfig::get_operator</a>(validator_account)
                <b>with</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>



</details>

<a name="0x1_ValidatorAdministrationScripts_set_validator_operator"></a>

## Function `set_validator_operator`


<a name="@Summary_20"></a>

### Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by an account with
Validator role.


<a name="@Technical_Description_21"></a>

### Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the sending validator account. The account at <code>operator_account</code> address must have
a Validator Operator role and have a <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code>
resource published under it. The sending <code>account</code> must be a Validator and have a
<code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. This script does not emit a
<code><a href="DiemConfig.md#0x1_DiemConfig_NewEpochEvent">DiemConfig::NewEpochEvent</a></code> and no reconfiguration of the system is initiated by this script.


<a name="@Parameters_22"></a>

### Parameters

| Name               | Type         | Description                                                                                  |
| ------             | ------       | -------------                                                                                |
| <code>account</code>          | <code>signer</code>     | The signer of the sending account of the transaction.                                        |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                             |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator. |


<a name="@Common_Abort_Conditions_23"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| 0                          | 0                                                     | The <code>human_name</code> field of the <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="@Related_Scripts_24"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">ValidatorAdministrationScripts::register_validator_config</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">ValidatorAdministrationScripts::remove_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">ValidatorAdministrationScripts::add_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">ValidatorAdministrationScripts::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">set_validator_operator</a>(account: signer, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">set_validator_operator</a>(
    account: signer,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <b>assert</b>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(&account, operator_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">ValidatorConfig::AbortsIfNoValidatorConfig</a>{addr: account_addr};
<b>aborts_if</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) != operator_name <b>with</b> 0;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetOperatorAbortsIf">ValidatorConfig::SetOperatorAbortsIf</a>{validator_account: account, operator_addr: operator_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetOperatorEnsures">ValidatorConfig::SetOperatorEnsures</a>{validator_account: account, operator_addr: operator_account};
</code></pre>


Reports INVALID_STATE because of !exists<DiemSystem::CapabilityHolder>, but that can't happen
because CapabilityHolder is published during initialization (Genesis).


<pre><code><b>aborts_with</b> [check]
    0, // Odd error code in <b>assert</b> on second statement in add_validator_and_reconfigure
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>


**Access Control:**
Only a Validator account can set its Validator Operator [[H16]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidator">Roles::AbortsIfNotValidator</a>{validator_addr: account_addr};
</code></pre>



</details>

<a name="0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin"></a>

## Function `set_validator_operator_with_nonce_admin`


<a name="@Summary_25"></a>

### Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by the Diem Root
account as a write set transaction.


<a name="@Technical_Description_26"></a>

### Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the validator <code>account</code>. The account at <code>operator_account</code> address must have a
Validator Operator role and have a <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource
published under it. The account represented by the <code>account</code> signer must be a Validator and
have a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. No reconfiguration of
the system is initiated by this script.


<a name="@Parameters_27"></a>

### Parameters

| Name               | Type         | Description                                                                                   |
| ------             | ------       | -------------                                                                                 |
| <code>dr_account</code>       | <code>signer</code>     | Signer of the sending account of the write set transaction. May only be the Diem Root signer. |
| <code>account</code>          | <code>signer</code>     | Signer of account specified in the <code>execute_as</code> field of the write set transaction.           |
| <code>sliding_nonce</code>    | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction for Diem Root.      |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                              |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator.  |


<a name="@Common_Abort_Conditions_28"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>                        | A <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> resource is not published under <code>dr_account</code>.                                                                                               |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                        | The <code>sliding_nonce</code> in <code>dr_account</code> is too old and it's impossible to determine if it's duplicated or not.                                                   |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                        | The <code>sliding_nonce</code> in <code>dr_account</code> is too far in the future.                                                                                                |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>               | The <code>sliding_nonce</code> in<code> dr_account</code> has been previously recorded.                                                                                            |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">SlidingNonce::ESLIDING_NONCE</a></code>                        | The sending account is not the Diem Root account or Treasury Compliance account                                                                             |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| 0                          | 0                                                     | The <code>human_name</code> field of the <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="@Related_Scripts_29"></a>

### Related Scripts

* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_account">AccountCreationScripts::create_validator_account</a></code>
* <code><a href="AccountCreationScripts.md#0x1_AccountCreationScripts_create_validator_operator_account">AccountCreationScripts::create_validator_operator_account</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_register_validator_config">ValidatorAdministrationScripts::register_validator_config</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_remove_validator_and_reconfigure">ValidatorAdministrationScripts::remove_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_add_validator_and_reconfigure">ValidatorAdministrationScripts::add_validator_and_reconfigure</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator">ValidatorAdministrationScripts::set_validator_operator</a></code>
* <code><a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_config_and_reconfigure">ValidatorAdministrationScripts::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(dr_account: signer, account: signer, sliding_nonce: u64, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ValidatorAdministrationScripts.md#0x1_ValidatorAdministrationScripts_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(
    dr_account: signer,
    account: signer,
    sliding_nonce: u64,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <a href="SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(&dr_account, sliding_nonce);
    <b>assert</b>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(&account, operator_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>include</b> <a href="DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{seq_nonce: sliding_nonce, account: dr_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">ValidatorConfig::AbortsIfNoValidatorConfig</a>{addr: account_addr};
<b>aborts_if</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) != operator_name <b>with</b> 0;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetOperatorAbortsIf">ValidatorConfig::SetOperatorAbortsIf</a>{validator_account: account, operator_addr: operator_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_SetOperatorEnsures">ValidatorConfig::SetOperatorEnsures</a>{validator_account: account, operator_addr: operator_account};
<b>aborts_with</b> [check]
    0, // Odd error code in <b>assert</b> on second statement in add_validator_and_reconfigure
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a>;
</code></pre>


**Access Control:**
Only the Diem Root account can process the admin scripts [[H9]][PERMISSION].


<pre><code><b>requires</b> <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(dr_account);
</code></pre>


This is ensured by DiemAccount::writeset_prologue.
Only a Validator account can set its Validator Operator [[H16]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidator">Roles::AbortsIfNotValidator</a>{validator_addr: account_addr};
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
