
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(payee: address, auth_key_prefix: vector&lt;u8&gt;, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(
    payee: address,
    auth_key_prefix: vector&lt;u8&gt;,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
  <b>if</b> (!<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_exists">LibraAccount::exists</a>(payee)) {
      <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;Token&gt;(payee, auth_key_prefix);
  };
  <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_pay_from_sender_with_metadata">LibraAccount::pay_from_sender_with_metadata</a>&lt;Token&gt;(payee, amount, metadata, metadata_signature)
}
</code></pre>



</details>
