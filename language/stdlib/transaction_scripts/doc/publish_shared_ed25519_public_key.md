
<a name="SCRIPT"></a>

# Script `publish_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `publish_shared_ed25519_public_key`](#SCRIPT_publish_shared_ed25519_public_key)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_publish_shared_ed25519_public_key"></a>

## Function `publish_shared_ed25519_public_key`


<a name="SCRIPT_@Summary"></a>

### Summary

Rotates the authentication key of the sending account to the
newly-specified public key and publishes a new shared authentication key
under the sender's account. Any account can send this transaction.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Rotates the authentication key of the sending account to <code>public_key</code>,
and publishes a <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource
containing the 32-byte ed25519 <code>public_key</code> and the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for
<code>account</code> under <code>account</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name         | Type         | Description                                                                               |
| ------       | ------       | -------------                                                                             |
| <code>account</code>    | <code>&signer</code>    | The signer reference of the sending account of the transaction.                           |
| <code>public_key</code> | <code>vector&lt;u8&gt;</code> | 32-byte Ed25519 public key for <code>account</code>' authentication key to be rotated to and stored. |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category              | Error Reason                                               | Description                                                                                         |
| ----------------            | --------------                                             | -------------                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> resource.       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_ESHARED_KEY">SharedEd25519PublicKey::ESHARED_KEY</a></code>                      | The <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource is already published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY</a></code>            | <code>public_key</code> is an invalid ed25519 public key.                                                      |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::rotate_shared_ed25519_public_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">SharedEd25519PublicKey::publish</a>(account, public_key)
}
</code></pre>



</details>
