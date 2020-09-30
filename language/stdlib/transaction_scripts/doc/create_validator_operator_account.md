
<a name="create_validator_operator_account"></a>

# Script `create_validator_operator_account`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Creates a Validator Operator account. This transaction can only be sent by the Libra
Root account.


<a name="@Technical_Description_1"></a>

## Technical Description

Creates an account with a Validator Operator role at <code>new_account_address</code>, with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>. It publishes a
<code><a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig">ValidatorOperatorConfig::ValidatorOperatorConfig</a></code> resource with the specified <code>human_name</code>.
This script does not assign the validator operator to any validator accounts but only creates the account.


<a name="@Parameters_2"></a>

## Parameters

| Name                  | Type         | Description                                                                                     |
| ------                | ------       | -------------                                                                                   |
| <code>lr_account</code>          | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
| <code>sliding_nonce</code>       | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                      |
| <code>new_account_address</code> | <code>address</code>    | Address of the to-be-created Validator account.                                                 |
| <code>auth_key_prefix</code>     | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.        |
| <code>human_name</code>          | <code>vector&lt;u8&gt;</code> | ASCII-encoded human name for the validator.                                                     |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category | Error Reason | Description |
|----------------|--------------|-------------|
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>  | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>            | The sending account is not the Libra Root account.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                       | The <code>new_account_address</code> address is already taken.                                        |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code><a href="create_validator_account.md#create_validator_account">Script::create_validator_account</a></code>
* <code><a href="add_validator_and_reconfigure.md#add_validator_and_reconfigure">Script::add_validator_and_reconfigure</a></code>
* <code><a href="register_validator_config.md#register_validator_config">Script::register_validator_config</a></code>
* <code><a href="remove_validator_and_reconfigure.md#remove_validator_and_reconfigure">Script::remove_validator_and_reconfigure</a></code>
* <code><a href="set_validator_operator.md#set_validator_operator">Script::set_validator_operator</a></code>
* <code><a href="set_validator_operator_with_nonce_admin.md#set_validator_operator_with_nonce_admin">Script::set_validator_operator_with_nonce_admin</a></code>
* <code><a href="set_validator_config_and_reconfigure.md#set_validator_config_and_reconfigure">Script::set_validator_config_and_reconfigure</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="create_validator_operator_account.md#create_validator_operator_account">create_validator_operator_account</a>(lr_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="create_validator_operator_account.md#create_validator_operator_account">create_validator_operator_account</a>(
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
