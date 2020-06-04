
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
  <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_pay_from_sender_with_metadata">LibraAccount::pay_from_sender_with_metadata</a>&lt;Token&gt;(payee, amount, metadata, metadata_signature)
}
</code></pre>



</details>
