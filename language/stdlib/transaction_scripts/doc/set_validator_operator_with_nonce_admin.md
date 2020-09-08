
<a name="SCRIPT"></a>

# Script `set_validator_operator_with_nonce_admin.move`

### Table of Contents

-  [Function `set_validator_operator_with_nonce_admin`](#SCRIPT_set_validator_operator_with_nonce_admin)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_set_validator_operator_with_nonce_admin"></a>

## Function `set_validator_operator_with_nonce_admin`


<a name="SCRIPT_@Summary"></a>

### Summary

Sets the validator operator for a validator in the validator's configuration resource "locally"
and does not reconfigure the system. Changes from this transaction will not picked up by the
system until a reconfiguration of the system is triggered. May only be sent by the Libra Root
account as a write set transaction.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Sets the account at <code>operator_account</code> address and with the specified <code>human_name</code> as an
operator for the validator <code>account</code>. The account at <code>operator_account</code> address must have a
Validator Operator role and have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource
published under it. The account represented by the <code>account</code> signer must be a Validator and
have a <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> resource published under it. No reconfiguration of
the system is initiated by this script.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name               | Type         | Description                                                                                                  |
| ------             | ------       | -------------                                                                                                |
| <code>lr_account</code>       | <code>&signer</code>    | The signer reference of the sending account of the write set transaction. May only be the Libra Root signer. |
| <code>account</code>          | <code>&signer</code>    | Signer reference of account specified in the <code>execute_as</code> field of the write set transaction.                |
| <code>sliding_nonce</code>    | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction for Libra Root.                    |
| <code>operator_name</code>    | <code>vector&lt;u8&gt;</code> | Validator operator's human name.                                                                             |
| <code>operator_account</code> | <code>address</code>    | Address of the validator operator account to be added as the <code>account</code> validator's operator.                 |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                          | Description                                                                                                                                                  |
| ----------------           | --------------                                        | -------------                                                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                        | The <code>sliding_nonce</code> in <code>lr_account</code> is too old and it's impossible to determine if it's duplicated or not.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                        | The <code>sliding_nonce</code> in <code>lr_account</code> is too far in the future.                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>               | The <code>sliding_nonce</code> in<code> lr_account</code> has been previously recorded.                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG</a></code> | The <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource is not published under <code>operator_account</code>.                                                   |
| EMPTY                      | 0                                                     | The <code>human_name</code> field of the <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource under <code>operator_account</code> does not match the provided <code>human_name</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>    | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EVALIDATOR">Roles::EVALIDATOR</a></code>                                   | <code>account</code> does not have a Validator account role.                                                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR</a></code>          | The account at <code>operator_account</code> does not have a <code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource.                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">ValidatorConfig::EVALIDATOR_CONFIG</a></code>                  | A <code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">ValidatorConfig::ValidatorConfig</a></code> is not published under <code>account</code>.                                                                                       |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_validator_account</code>
* <code>Script::create_validator_operator_account</code>
* <code>Script::register_validator_config</code>
* <code>Script::remove_validator_and_reconfigure</code>
* <code>Script::add_validator_and_reconfigure</code>
* <code>Script::set_validator_operator</code>
* <code>Script::set_validator_config_and_reconfigure</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(
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
