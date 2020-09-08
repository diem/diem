
<a name="SCRIPT"></a>

# Script `rotate_authentication_key.move`

### Table of Contents

-  [Function `rotate_authentication_key`](#SCRIPT_rotate_authentication_key)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `rotate_authentication_key`](#SCRIPT_Specification_rotate_authentication_key)



<a name="SCRIPT_rotate_authentication_key"></a>

## Function `rotate_authentication_key`


<a name="SCRIPT_@Summary"></a>

### Summary

Rotates the transaction sender's authentication key to the supplied new authentication key. May
be sent by any account.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Rotate the <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>authentication_key</code> field to <code>new_key</code>.
<code>new_key</code> must be a valid ed25519 public key, and <code>account</code> must not have previously delegated
its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name      | Type         | Description                                                 |
| ------    | ------       | -------------                                               |
| <code>account</code> | <code>&signer</code>    | Signer reference of the sending account of the transaction. |
| <code>new_key</code> | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for <code>account</code>.            |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                              |
| ----------------           | --------------                                             | -------------                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">LibraAccount::EMALFORMED_AUTHENTICATION_KEY</a></code>              | <code>new_key</code> was an invalid length.                                                         |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::rotate_authentication_key_with_nonce</code>
* <code>Script::rotate_authentication_key_with_nonce_admin</code>
* <code>Script::rotate_authentication_key_with_recovery_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;) {
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>




<a name="SCRIPT_account_addr$1"></a>


<pre><code><b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<a name="SCRIPT_key_rotation_capability$2"></a>
<b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">LibraAccount::RotateAuthenticationKeyAbortsIf</a>{cap: key_rotation_capability, new_authentication_key: new_key};
</code></pre>


This rotates the authentication key of <code>account</code> to <code>new_key</code>


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">LibraAccount::RotateAuthenticationKeyEnsures</a>{addr: account_addr, new_authentication_key: new_key};
</code></pre>
