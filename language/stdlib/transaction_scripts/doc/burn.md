
<a name="SCRIPT"></a>

# Script `burn.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(sliding_nonce: u64, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(sliding_nonce: u64, preburn_address: address) {
    <a href="../../modules/doc/sliding_nonce.md#0x0_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(sliding_nonce);
    <a href="../../modules/doc/libra.md#0x0_Libra_burn">Libra::burn</a>&lt;Token&gt;(preburn_address)
}
</code></pre>



</details>
