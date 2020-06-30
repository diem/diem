
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`

Transfer
<code>amount</code> coins to
<code>recipient_address</code> with (optional)
associated metadata
<code>metadata</code> and (optional)
<code>signature</code> on the metadata, amount, and
sender address. The
<code>metadata</code> and
<code>signature</code> parameters are only required if
<code>amount</code> >= 1_000_000 micro LBR and the sender and recipient of the funds are two distinct VASPs.
Fails if there is no account at the recipient address or if the sender's balance is lower
than
<code>amount</code>.


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
