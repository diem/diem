
<a name="SCRIPT"></a>

# Script `create_child_vasp_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

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


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool, child_initial_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_create_child_vasp_account">LibraAccount::create_child_vasp_account</a>&lt;CoinType&gt;(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    <b>if</b> (child_initial_balance &gt; 0) {
        <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;CoinType&gt;(parent_vasp, child_address, child_initial_balance)
    };
}
</code></pre>



</details>
