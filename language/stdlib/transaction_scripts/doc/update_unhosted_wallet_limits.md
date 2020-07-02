
<a name="SCRIPT"></a>

# Script `update_unhosted_wallet_limits.move`

### Table of Contents

-  [Function `update_unhosted_wallet_limits`](#SCRIPT_update_unhosted_wallet_limits)



<a name="SCRIPT_update_unhosted_wallet_limits"></a>

## Function `update_unhosted_wallet_limits`

Optionally update global thresholds of max balance, total flow (inflow + outflow) (microLBR)
for
<code>LimitsDefinition</code> bound accounts.
If a new threshold is 0, that particular config does not get updated.
<code>sliding_nonce</code> is a unique nonce for operation, see sliding_nonce.move for details.


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
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_update_limits_definition">AccountLimits::update_limits_definition</a>(tc_account, new_max_total_flow, new_max_holding_balance);
}
</code></pre>



</details>
