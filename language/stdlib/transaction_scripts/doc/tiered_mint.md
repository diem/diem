
<a name="SCRIPT"></a>

# Script `tiered_mint.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Script for Treasury Comliance Account to mint 'mint_amount' to 'designated_dealer_address'
for 'tier_index' tier


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(tc_account: &signer, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(tc_account: &signer, designated_dealer_address: address, mint_amount: u64, tier_index: u64) {
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_mint_to_designated_dealer">LibraAccount::mint_to_designated_dealer</a>&lt;CoinType&gt;(tc_account, designated_dealer_address, mint_amount, tier_index);
}
</code></pre>



</details>
