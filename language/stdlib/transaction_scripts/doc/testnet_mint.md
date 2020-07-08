
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
  // Each mint amount must be under the dual attestation threshold. This is because the "minter" is
  // is a <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a> account, and the recipient (at least in the current testnet) will always
  // be a <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a> or <a href="../../modules/doc/VASP.md#0x1_VASP">VASP</a>.
  <b>assert</b>(
      <a href="../../modules/doc/Libra.md#0x1_Libra_approx_lbr_for_value">Libra::approx_lbr_for_value</a>&lt;Token&gt;(amount) &lt; <a href="../../modules/doc/DualAttestationLimit.md#0x1_DualAttestationLimit_get_cur_microlibra_limit">DualAttestationLimit::get_cur_microlibra_limit</a>(),
      8000973
  );
  <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;Token&gt;(&payer_withdrawal_cap, payee, amount, x"", x"");
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>
