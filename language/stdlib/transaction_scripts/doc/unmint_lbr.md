
<a name="SCRIPT"></a>

# Script `unmint_lbr.move`

### Table of Contents

-  [Function `unmint_lbr`](#SCRIPT_unmint_lbr)



<a name="SCRIPT_unmint_lbr"></a>

## Function `unmint_lbr`

Unmints
<code>amount_lbr</code> LBR from the sending account into the constituent coins and deposits
the resulting coins into the sending account."


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <b>let</b> lbr = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>&lt;<a href="../../modules/doc/LBR.md#0x1_LBR">LBR</a>&gt;(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
    <b>let</b> (coin1, coin2) = <a href="../../modules/doc/LBR.md#0x1_LBR_unpack">LBR::unpack</a>(account, lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_deposit_to">LibraAccount::deposit_to</a>(account, coin1);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_deposit_to">LibraAccount::deposit_to</a>(account, coin2);
}
</code></pre>



</details>
