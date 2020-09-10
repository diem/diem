
<a name="SCRIPT"></a>

# Script `publish_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `publish_shared_ed25519_public_key`](#SCRIPT_publish_shared_ed25519_public_key)



<a name="SCRIPT_publish_shared_ed25519_public_key"></a>

## Function `publish_shared_ed25519_public_key`

(1) Rotate the authentication key of the sender to
<code>public_key</code>
(2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
of the sender under the sender's address.
Aborts if the sender already has a
<code><a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.
Aborts if the length of
<code>new_public_key</code> is not 32.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">SharedEd25519PublicKey::publish</a>(account, public_key)
}
</code></pre>



</details>
