
<a name="SCRIPT"></a>

# Script `testnet_mint.move`

### Table of Contents

-  [Function `testnet_mint`](#SCRIPT_testnet_mint)



<a name="SCRIPT_testnet_mint"></a>

## Function `testnet_mint`

Send
<code>amount</code> coins of type
<code>Token</code> to
<code>payee</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_testnet_mint">testnet_mint</a>&lt;Token&gt;(payer: &signer, payee: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_testnet_mint">testnet_mint</a>&lt;Token&gt;(payer: &signer, payee: address, amount: u64) {
  <b>assert</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_exists_at">LibraAccount::exists_at</a>(payee), 8000971);
  <b>assert</b>(<a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(payer) == 0xDD, 8000972);
  // Limit minting <b>to</b> 1B <a href="../../modules/doc/Libra.md#0x1_Libra">Libra</a> at a time on testnet.
  // This is <b>to</b> prevent the market cap's total value from hitting u64_max due <b>to</b> excessive
  // minting. This will not be a problem in the production <a href="../../modules/doc/Libra.md#0x1_Libra">Libra</a> system because coins will
  // be backed with real-world assets, and thus minting will be correspondingly rarer.
  // * 1000000 here because the unit is microlibra
  <b>assert</b>(amount &lt;= 1000000000 * 1000000, 8000973);
  <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from_with_metadata">LibraAccount::pay_from_with_metadata</a>&lt;Token&gt;(&payer_withdrawal_cap, payee, amount, x"", x"");
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>
