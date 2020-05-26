
<a name="SCRIPT"></a>

# Script `create_child_vasp_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_create_child_vasp_account">LibraAccount::create_child_vasp_account</a>&lt;CoinType&gt;(child_address, auth_key_prefix, parent_vasp)
}
</code></pre>



</details>
