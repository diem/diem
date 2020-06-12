
<a name="SCRIPT"></a>

# Script `burn_txn_fees.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(blessed_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(blessed_account: &signer) {
    <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_preburn_fees">TransactionFee::preburn_fees</a>&lt;CoinType&gt;(blessed_account);
    <b>if</b> (<a href="../../modules/doc/LBR.md#0x1_LBR_is_lbr">LBR::is_lbr</a>&lt;CoinType&gt;()) {
        <b>let</b> coin1_burn_cap = <a href="../../modules/doc/Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="../../modules/doc/Coin1.md#0x1_Coin1">Coin1</a>&gt;(blessed_account);
        <b>let</b> coin2_burn_cap = <a href="../../modules/doc/Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="../../modules/doc/Coin2.md#0x1_Coin2">Coin2</a>&gt;(blessed_account);
        <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>(&coin1_burn_cap);
        <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>(&coin2_burn_cap);
        <a href="../../modules/doc/Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(blessed_account, coin1_burn_cap);
        <a href="../../modules/doc/Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(blessed_account, coin2_burn_cap);
    } <b>else</b> {
        <b>let</b> burn_cap = <a href="../../modules/doc/Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;CoinType&gt;(blessed_account);
        <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>(&burn_cap);
        <a href="../../modules/doc/Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(blessed_account, burn_cap);
    }
}
</code></pre>



</details>
