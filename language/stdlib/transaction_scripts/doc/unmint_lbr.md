
<a name="SCRIPT"></a>

# Script `unmint_lbr.move`

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
    <b>let</b> lbr = <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>&lt;<a href="../../modules/doc/lbr.md#0x0_LBR_T">LBR::T</a>&gt;(amount_lbr);
    <b>let</b> (coin1, coin2) = <a href="../../modules/doc/lbr.md#0x0_LBR_unpack">LBR::unpack</a>(lbr);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(sender, coin1);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(sender, coin2);
}
</code></pre>



</details>
