
<a name="SCRIPT"></a>

# Script `set_validator_config_and_reconfigure.move`

### Table of Contents

-  [Function `set_validator_config_and_reconfigure`](#SCRIPT_set_validator_config_and_reconfigure)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_set_validator_config_and_reconfigure"></a>

## Function `set_validator_config_and_reconfigure`


<a name="SCRIPT_@Summary"></a>

### Summary

Updates a validator's configuration, and triggers a reconfiguration of the system to update the
validator set with this new validator configuration.  Can only be successfully sent by a
Validator Operator account that is already registered with a validator.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

This updates the fields with corresponding names held in the <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>
config resource held under <code>validator_account</code>. It then emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> to
trigger a reconfiguration of the system.  This reconfiguration will update the validator set
on-chain with the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                          | Type         | Description                                                                                                                  |
| ------                        | ------       | -------------                                                                                                                |
| <code>validator_operator_account</code>  | <code>&signer</code>    | Signer reference of the sending account. Must be the registered validator operator for the validator at <code>validator_address</code>. |
| <code>validator_account</code>           | <code>address</code>    | The address of the validator's <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource being updated.                                    |
| <code>consensus_pubkey</code>            | <code>vector&lt;u8&gt;</code> | New Ed25519 public key to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                                         |
| <code>validator_network_addresses</code> | <code>vector&lt;u8&gt;</code> | New set of <code>validator_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                       |
| <code>fullnode_network_addresses</code>  | <code>vector&lt;u8&gt;</code> | New set of <code>fullnode_network_addresses</code> to be used in the updated <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code>.                        |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                   | Description                                                                                           |
| ----------------           | --------------                                 | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>           | <code>validator_address</code> does not have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it.   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">ValidatorConfig::EINVALID_TRANSACTION_SENDER</a></code> | <code>validator_operator_account</code> is not the registered operator for the validator at <code>validator_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">ValidatorConfig::EINVALID_CONSENSUS_KEY</a></code>      | <code>consensus_pubkey</code> is not a valid ed25519 public key.                                                 |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_validator_account</code>
* <code>Script::create_validator_operator_account</code>
* <code>Script::add_validator_and_reconfigure</code>
* <code>Script::remove_validator_and_reconfigure</code>
* <code>Script::set_validator_operator</code>
* <code>Script::set_validator_operator_with_nonce_admin</code>
* <code>Script::register_validator_config</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(validator_operator_account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(
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
