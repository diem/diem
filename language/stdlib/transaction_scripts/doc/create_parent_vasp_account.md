
<a name="SCRIPT"></a>

# Script `create_parent_vasp_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(association: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(
    association: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">LibraAccount::create_parent_vasp_account</a>&lt;CoinType&gt;(
        association,
        new_account_address,
        auth_key_prefix,
        human_name,
        base_url,
        compliance_public_key,
        add_all_currencies
    )
}
</code></pre>



</details>
