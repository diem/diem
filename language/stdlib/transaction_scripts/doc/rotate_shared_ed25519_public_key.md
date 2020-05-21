
<a name="SCRIPT"></a>

# Script `rotate_shared_ed25519_public_key.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/shared_ed25519_public_key.md#0x0_SharedEd25519PublicKey_rotate_sender_key">SharedEd25519PublicKey::rotate_sender_key</a>(public_key)
}
</code></pre>



</details>
