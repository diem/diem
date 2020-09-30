
<a name="set_validator_operator"></a>

# Script `set_validator_operator`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by an account with
Validator role.


<a name="@Technical_Description_1"></a>

## Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the sending validator account. The account at <code>operator_account</code> address must have
a Validator Operator role and have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code>
resource published under it. The sending <code>account</code> must be a Validator and have a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. This script does not emit a
<code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> and no reconfiguration of the system is initiated by this script.


<a name="@Parameters_2"></a>

## Parameters

| Name               | Type         | Description                                                                                  |
| ------             | ------       | -------------                                                                                |
| <code>account</code>          | <code>&signer</code>    | The signer reference of the sending account of the transaction.                              |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                             |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator. |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| EMPTY                      | 0                                                     | The <code>human_name</code> field of the <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code><a href="create_validator_account.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="create_validator_operator_account.md#create_validator_operator_account">Script::create_validator_operator_account</a></code>
* <code><a href="register_validator_config.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="remove_validator_and_reconfigure.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="add_validator_and_reconfigure.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="set_validator_operator_with_nonce_admin.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="set_validator_config_and_reconfigure.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="set_validator_operator.md#set_validator_operator">set_validator_operator</a>(account: &signer, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="set_validator_operator.md#set_validator_operator">set_validator_operator</a>(
    account: &signer,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <b>assert</b>(<a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(account, operator_account);
}
</code></pre>



</details>
