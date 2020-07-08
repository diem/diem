
<a name="SCRIPT"></a>

# Script `update_account_limit_window_info.move`

### Table of Contents

-  [Function `update_account_limit_window_info`](#SCRIPT_update_account_limit_window_info)



<a name="SCRIPT_update_account_limit_window_info"></a>

## Function `update_account_limit_window_info`

* Sets the account limits window
<code>tracking_balance</code> field for
<code>CoinType</code> at
<code>window_address</code> to
<code>aggregate_balance</code> if
<code>aggregate_balance != 0</code>.
* Sets the account limits window
<code>limit_address</code> field for
<code>CoinType</code> at
<code>window_address</code> to
<code>new_limit_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_account_limit_window_info">update_account_limit_window_info</a>&lt;CoinType&gt;(tc_account: &signer, window_address: address, aggregate_balance: u64, new_limit_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_account_limit_window_info">update_account_limit_window_info</a>&lt;CoinType&gt;(
    tc_account: &signer,
    window_address: address,
    aggregate_balance: u64,
    new_limit_address: address
) {
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_update_window_info">AccountLimits::update_window_info</a>&lt;CoinType&gt;(tc_account, window_address, aggregate_balance, new_limit_address);
}
</code></pre>



</details>
