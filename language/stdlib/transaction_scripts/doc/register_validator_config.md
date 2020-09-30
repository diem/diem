
<a name="register_validator_config"></a>

# Script `register_validator_config`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Updates a validator's configuration. This does not reconfigure the system and will not update
the configuration in the validator set that is seen by other validators in the network. Can
only be successfully sent by a Validator Operator account that is already registered with a
validator.


<a name="@Technical_Description_1"></a>

## Technical Description

This updates the fields with corresponding names held in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It does not emit a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code>
so the copy of this config held in the validator set will not be updated, and the changes are
only "locally" under the <code>validator_account</code> account address.


<a name="@Parameters_2"></a>

## Parameters

| Name                          | Type         | Description                                                                                                                  |
| ------                        | ------       | -------------                                                                                                                |
| <code>validator_operator_account</code>  | <code>&signer</code>    | Signer reference of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                                    |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                                         |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                       |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                        |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code><a href="create_validator_account.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="create_validator_operator_account.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="add_validator_and_reconfigure.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="remove_validator_and_reconfigure.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="set_validator_operator.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="set_validator_operator_with_nonce_admin.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="set_validator_config_and_reconfigure.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="register_validator_config.md#register_validator_config">register_validator_config</a>(validator_operator_account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="register_validator_config.md#register_validator_config">register_validator_config</a>(
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
 }
</code></pre>



</details>
