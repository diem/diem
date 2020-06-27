
<a name="SCRIPT"></a>

# Script `mint_lbr.move`

### Table of Contents

-  [Function `mint_lbr`](#SCRIPT_mint_lbr)



<a name="SCRIPT_mint_lbr"></a>

## Function `mint_lbr`

Mint
<code>amount_lbr</code> LBR from the sending account's constituent coins and deposits the
resulting LBR into the sending account.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_staple_lbr">LibraAccount::staple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap)
}
</code></pre>



</details>
