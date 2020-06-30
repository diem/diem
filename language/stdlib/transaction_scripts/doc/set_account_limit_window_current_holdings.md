
<a name="SCRIPT"></a>

# Script `set_account_limit_window_current_holdings.move`

### Table of Contents

-  [Function `set_account_limit_window_current_holdings`](#SCRIPT_set_account_limit_window_current_holdings)



<a name="SCRIPT_set_account_limit_window_current_holdings"></a>

## Function `set_account_limit_window_current_holdings`

Sets the account limits window
<code>tracking_balance</code> field for
<code>CointType</code> at
<code>window_address</code> to
<code>aggregate_balance</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_account_limit_window_current_holdings">set_account_limit_window_current_holdings</a>&lt;CointType&gt;(tc_account: &signer, window_address: address, aggregate_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_account_limit_window_current_holdings">set_account_limit_window_current_holdings</a>&lt;CointType&gt;(tc_account: &signer,  window_address: address, aggregate_balance: u64) {
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_set_current_holdings">AccountLimits::set_current_holdings</a>&lt;CointType&gt;(tc_account, window_address, aggregate_balance);
}
</code></pre>



</details>
