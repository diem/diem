
<a name="SCRIPT"></a>

# Script `create_child_vasp_account.move`

### Table of Contents

-  [Function `create_child_vasp_account`](#SCRIPT_create_child_vasp_account)



<a name="SCRIPT_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create a
<code>ChildVASP</code> account for sender
<code>parent_vasp</code> at
<code>child_address</code> with a balance of
<code>child_initial_balance</code> in
<code>CoinType</code> and an initial authentication_key
<code>auth_key_prefix | child_address</code>.
If
<code>add_all_currencies</code> is true, the child address will have a zero balance in all available
currencies in the system


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool, child_initial_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_child_vasp_account">LibraAccount::create_child_vasp_account</a>&lt;CoinType&gt;(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    <b>if</b> (child_initial_balance &gt; 0) {
        <b>let</b> vasp_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(parent_vasp);
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;CoinType&gt;(&vasp_withdrawal_cap, child_address, child_initial_balance);
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(vasp_withdrawal_cap);
    };
}
</code></pre>



</details>
