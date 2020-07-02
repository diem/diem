
<a name="SCRIPT"></a>

# Script `create_parent_vasp_account.move`

### Table of Contents

-  [Function `create_parent_vasp_account`](#SCRIPT_create_parent_vasp_account)



<a name="SCRIPT_create_parent_vasp_account"></a>

## Function `create_parent_vasp_account`

Create an account with the ParentVASP role at
<code>address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code> and a 0 balance of type
<code>currency</code>. If
<code>add_all_currencies</code> is true, 0 balances for all available currencies in the system will
also be added. This can only be invoked by an Association account.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_parent_vasp_account">create_parent_vasp_account</a>&lt;CoinType&gt;(lr_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_parent_vasp_account">create_parent_vasp_account</a>&lt;CoinType&gt;(
    lr_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">LibraAccount::create_parent_vasp_account</a>&lt;CoinType&gt;(
        lr_account,
        new_account_address,
        auth_key_prefix,
        human_name,
        base_url,
        compliance_public_key,
        add_all_currencies
    );
}
</code></pre>



</details>
