
<a name="SCRIPT"></a>

# Script `publish_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `publish_shared_ed25519_public_key`](#SCRIPT_publish_shared_ed25519_public_key)



<a name="SCRIPT_publish_shared_ed25519_public_key"></a>

## Function `publish_shared_ed25519_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_publish_shared_ed25519_public_key">publish_shared_ed25519_public_key</a>(account: &signer, public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">SharedEd25519PublicKey::publish</a>(account, public_key)
}
</code></pre>



</details>
