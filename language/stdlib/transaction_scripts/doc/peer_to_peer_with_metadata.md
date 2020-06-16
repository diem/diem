
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Token&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Token&gt;(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
  <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from_with_metadata">LibraAccount::pay_from_with_metadata</a>&lt;Token&gt;(&payer_withdrawal_cap, payee, amount, metadata, metadata_signature);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>
