
<a name="SCRIPT"></a>

# Script `update_unhosted_wallet_limits.move`

### Table of Contents

-  [Function `update_unhosted_wallet_limits`](#SCRIPT_update_unhosted_wallet_limits)



<a name="SCRIPT_update_unhosted_wallet_limits"></a>

## Function `update_unhosted_wallet_limits`

Script for Treasury Comliance Account to optionally update global thresholds
of max balance, total flow (inflow + outflow) (microLBR) for LimitsDefinition bound accounts.
If the new threshold is zero, that particular config does not get updated.
sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_unhosted_wallet_limits">update_unhosted_wallet_limits</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_max_total_flow: u64, new_max_holding_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_unhosted_wallet_limits">update_unhosted_wallet_limits</a>&lt;CoinType&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_max_total_flow: u64,
    new_max_holding_balance: u64,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> cap = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;TreasuryComplianceRole&gt;(tc_account);
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_update_limits_definition">AccountLimits::update_limits_definition</a>(&cap, new_max_total_flow, new_max_holding_balance);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, cap);
}
</code></pre>



</details>
