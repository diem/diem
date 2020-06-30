
<a name="SCRIPT"></a>

# Script `update_account_limit_definition.move`

### Table of Contents

-  [Function `update_account_limit_definition`](#SCRIPT_update_account_limit_definition)



<a name="SCRIPT_update_account_limit_definition"></a>

## Function `update_account_limit_definition`

Optionally update thresholds of max balance, inflow, outflow
for any limits-bound accounts with their limits defined at
<code>limit_address</code>.
Limits are defined in terms of base (on-chain) currency units for
<code>CoinType</code>.
If a new threshold is 0, that particular config does not get updated.
<code>sliding_nonce</code> is a unique nonce for operation, see SlidingNonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_account_limit_definition">update_account_limit_definition</a>&lt;CoinType&gt;(tc_account: &signer, limit_address: address, sliding_nonce: u64, new_max_inflow: u64, new_max_outflow: u64, new_max_holding_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_account_limit_definition">update_account_limit_definition</a>&lt;CoinType&gt;(
    tc_account: &signer,
    limit_address: address,
    sliding_nonce: u64,
    new_max_inflow: u64,
    new_max_outflow: u64,
    new_max_holding_balance: u64,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_update_limits_definition">AccountLimits::update_limits_definition</a>&lt;CoinType&gt;(
        tc_account,
        limit_address,
        new_max_inflow,
        new_max_outflow,
        new_max_holding_balance
    );
}
</code></pre>



</details>
