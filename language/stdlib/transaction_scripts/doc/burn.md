
<a name="SCRIPT"></a>

# Script `burn.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Permanently destroy the
<code>Token</code>s stored in the oldest burn request under the
<code>Preburn</code> resource.
This will only succeed if
<code>account</code> has a
<code>MintCapability&lt;Token&gt;</code>, a
<code>Preburn&lt;Token&gt;</code> resource
exists under
<code>preburn_address</code>, and there is a pending burn request.
sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(account: &signer, sliding_nonce: u64, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(account: &signer, sliding_nonce: u64, preburn_address: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/Libra.md#0x1_Libra_burn">Libra::burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>
