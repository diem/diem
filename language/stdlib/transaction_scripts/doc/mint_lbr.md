
<a name="SCRIPT"></a>

# Script `mint_lbr.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> sender = <a href="../../modules/doc/Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> coin1_balance = <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="../../modules/doc/Coin1.md#0x0_Coin1">Coin1</a>&gt;(sender);
    <b>let</b> coin2_balance = <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="../../modules/doc/Coin2.md#0x0_Coin2">Coin2</a>&gt;(sender);
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <b>let</b> coin1 = <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>&lt;<a href="../../modules/doc/Coin1.md#0x0_Coin1">Coin1</a>&gt;(&withdraw_cap, coin1_balance);
    <b>let</b> coin2 = <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>&lt;<a href="../../modules/doc/Coin2.md#0x0_Coin2">Coin2</a>&gt;(&withdraw_cap, coin2_balance);
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
    <b>let</b> (lbr, coin1, coin2) = <a href="../../modules/doc/LBR.md#0x0_LBR_create">LBR::create</a>(amount_lbr, coin1, coin2);
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_deposit_to">LibraAccount::deposit_to</a>(account, lbr);
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_deposit_to">LibraAccount::deposit_to</a>(account, coin1);
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_deposit_to">LibraAccount::deposit_to</a>(account, coin2);
}
</code></pre>



</details>
