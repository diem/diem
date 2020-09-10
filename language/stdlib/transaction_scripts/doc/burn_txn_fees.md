
<a name="SCRIPT"></a>

# Script `burn_txn_fees.move`

### Table of Contents

-  [Function `burn_txn_fees`](#SCRIPT_burn_txn_fees)



<a name="SCRIPT_burn_txn_fees"></a>

## Function `burn_txn_fees`

Burn transaction fees that have been collected in the given
<code>currency</code>
and relinquish to the association. The currency must be non-synthetic.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer) {
    <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>&lt;CoinType&gt;(tc_account);
}
</code></pre>



</details>
