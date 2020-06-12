
<a name="SCRIPT"></a>

# Script `tiered_mint.move`

### Table of Contents

-  [Function `tiered_mint`](#SCRIPT_tiered_mint)



<a name="SCRIPT_tiered_mint"></a>

## Function `tiered_mint`

Script for Treasury Comliance Account to mint 'mint_amount' to 'designated_dealer_address' for
'tier_index' tier
sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details


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
    <b>let</b> tc_capability = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;TreasuryComplianceRole&gt;(tc_account);
    <b>let</b> coins = <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">DesignatedDealer::tiered_mint</a>&lt;CoinType&gt;(
        tc_account, &tc_capability, mint_amount, designated_dealer_address, tier_index
    );
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, tc_capability);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_deposit">LibraAccount::deposit</a>(tc_account, designated_dealer_address, coins)
}
</code></pre>



</details>
