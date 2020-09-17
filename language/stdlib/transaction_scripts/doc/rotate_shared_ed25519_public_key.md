
<a name="SCRIPT"></a>

# Script `rotate_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `rotate_shared_ed25519_public_key`](#SCRIPT_rotate_shared_ed25519_public_key)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_rotate_shared_ed25519_public_key"></a>

## Function `rotate_shared_ed25519_public_key`


<a name="SCRIPT_@Summary"></a>

### Summary

Rotates the authentication key in a <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code>. This transaction can be sent by
any account that has previously published a shared ed25519 public key using
<code>Script::publish_shared_ed25519_public_key</code>.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

This first rotates the public key stored in <code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource to <code>public_key</code>, after which it
rotates the authentication key using the capability stored in <code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> to a new value derived from <code>public_key</code>


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name         | Type         | Description                                                     |
| ------       | ------       | -------------                                                   |
| <code>account</code>    | <code>&signer</code>    | The signer reference of the sending account of the transaction. |
| <code>public_key</code> | <code>vector&lt;u8&gt;</code> | 32-byte Ed25519 public key.                                     |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                    | Description                                                                                   |
| ----------------           | --------------                                  | -------------                                                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_ESHARED_KEY">SharedEd25519PublicKey::ESHARED_KEY</a></code>           | A <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a></code> resource is not published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY</a></code> | <code>public_key</code> is an invalid ed25519 public key.                                                |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::publish_shared_ed25519_public_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">SharedEd25519PublicKey::rotate_key</a>(account, public_key)
}
</code></pre>



</details>
