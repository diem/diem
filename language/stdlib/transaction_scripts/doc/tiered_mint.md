
<a name="SCRIPT"></a>

# Script `tiered_mint.move`

### Table of Contents

-  [Function `tiered_mint`](#SCRIPT_tiered_mint)
-  [Specification](#SCRIPT_Specification)
    -  [Function `tiered_mint`](#SCRIPT_Specification_tiered_mint)



<a name="SCRIPT_tiered_mint"></a>

## Function `tiered_mint`

Mint
<code>mint_amount</code> to
<code>designated_dealer_address</code> for
<code>tier_index</code> tier.
Max valid tier index is 3 since there are max 4 tiers per DD.
Sender should be treasury compliance account and receiver authorized DD.
<code>sliding_nonce</code> is a unique nonce for operation, see sliding_nonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_tiered_mint">LibraAccount::tiered_mint</a>&lt;CoinType&gt;(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_tiered_mint"></a>

### Function `tiered_mint`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>




<pre><code><b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{account: tc_account, seq_nonce: sliding_nonce};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintAbortsIf">LibraAccount::TieredMintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">LibraAccount::TieredMintEnsures</a>&lt;CoinType&gt;;
</code></pre>
