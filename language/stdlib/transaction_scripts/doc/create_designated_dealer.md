
<a name="SCRIPT"></a>

# Script `create_designated_dealer.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Script for Treasury Compliance Account to create designated dealer account at 'new_account_address'
and 'auth_key_prefix' for nonsynthetic CoinType


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(tc_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(tc_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_create_designated_dealer">LibraAccount::create_designated_dealer</a>&lt;CoinType&gt;(tc_account, new_account_address, auth_key_prefix);
}
</code></pre>



</details>
