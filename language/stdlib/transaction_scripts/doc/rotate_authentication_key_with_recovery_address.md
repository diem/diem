
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_recovery_address.move`

### Table of Contents

-  [Function `rotate_authentication_key_with_recovery_address`](#SCRIPT_rotate_authentication_key_with_recovery_address)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `rotate_authentication_key_with_recovery_address`](#SCRIPT_Specification_rotate_authentication_key_with_recovery_address)



<a name="SCRIPT_rotate_authentication_key_with_recovery_address"></a>

## Function `rotate_authentication_key_with_recovery_address`


<a name="SCRIPT_@Summary"></a>

### Summary

Rotates the authentication key of a specified account that is part of a recovery address to a
new authentication key. Only used for accounts that are part of a recovery address (see
<code>Script::add_recovery_rotation_capability</code> for account restrictions).


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Rotates the authentication key of the <code>to_recover</code> account to <code>new_key</code> using the
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> stored in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource
published under <code>recovery_address</code>. This transaction can be sent either by the <code>to_recover</code>
account, or by the account where the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource is published
that contains <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name               | Type         | Description                                                                                                                    |
| ------             | ------       | -------------                                                                                                                  |
| <code>account</code>          | <code>&signer</code>    | Signer reference of the sending account of the transaction.                                                                    |
| <code>recovery_address</code> | <code>address</code>    | Address where <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> that holds <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> is published. |
| <code>to_recover</code>       | <code>address</code>    | The address of the account whose authentication key will be updated.                                                           |
| <code>new_key</code>          | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for the account at the <code>to_recover</code> address.                                                 |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                  | Description                                                                                                                                          |
| ----------------           | --------------                                | -------------                                                                                                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>          | <code>recovery_address</code> does not have a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource published under it.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY">RecoveryAddress::ECANNOT_ROTATE_KEY</a></code>         | The address of <code>account</code> is not <code>recovery_address</code> or <code>to_recover</code>.                                                                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE">RecoveryAddress::EACCOUNT_NOT_RECOVERABLE</a></code>   | <code>to_recover</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>  is not in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code>  resource published under <code>recovery_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code> | <code>new_key</code> was an invalid length.                                                                                                                     |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::rotate_authentication_key</code>
* <code>Script::rotate_authentication_key_with_nonce</code>
* <code>Script::rotate_authentication_key_with_nonce_admin</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(
    account: &signer,
    recovery_address: address,
    to_recover: address,
    new_key: vector&lt;u8&gt;
) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">RecoveryAddress::rotate_authentication_key</a>(account, recovery_address, to_recover, new_key)
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_rotate_authentication_key_with_recovery_address"></a>

### Function `rotate_authentication_key_with_recovery_address`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyAbortsIf">RecoveryAddress::RotateAuthenticationKeyAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyEnsures">RecoveryAddress::RotateAuthenticationKeyEnsures</a>;
</code></pre>
