
<a name="SCRIPT"></a>

# Script `rotate_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `rotate_shared_ed25519_public_key`](#SCRIPT_rotate_shared_ed25519_public_key)



<a name="SCRIPT_rotate_shared_ed25519_public_key"></a>

## Function `rotate_shared_ed25519_public_key`

(1) Rotate the public key stored in
<code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource to
<code>new_public_key</code>
(2) Rotate the authentication key using the capability stored in
<code>account</code>'s
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> to a new value derived from
<code>new_public_key</code>
Aborts if
<code>account</code> does not have a
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.
Aborts if the length of
<code>new_public_key</code> is not 32.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_shared_ed25519_public_key">rotate_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">SharedEd25519PublicKey::rotate_key</a>(account, public_key)
}
</code></pre>



</details>
