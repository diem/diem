
<a name="SCRIPT"></a>

# Script `mint_lbr.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(amount_lbr: u64) {
    <b>let</b> sender = Transaction::sender();
    <b>let</b> coin1_balance = <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="../../modules/doc/coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(sender);
    <b>let</b> coin2_balance = <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="../../modules/doc/coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(sender);
    <b>let</b> coin1 = <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>&lt;<a href="../../modules/doc/coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(coin1_balance);
    <b>let</b> coin2 = <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>&lt;<a href="../../modules/doc/coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(coin2_balance);
    <b>let</b> (lbr, coin1, coin2) = <a href="../../modules/doc/lbr.md#0x0_LBR_create">LBR::create</a>(amount_lbr, coin1, coin2);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(sender, lbr);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(sender, coin1);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(sender, coin2);
}
</code></pre>



</details>
