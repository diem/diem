
<a name="SCRIPT"></a>

# Script `create_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(fresh_address: address, auth_key_prefix: vector&lt;u8&gt;, initial_amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(fresh_address: address, auth_key_prefix: vector&lt;u8&gt;, initial_amount: u64) {
  <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;Token&gt;(fresh_address, auth_key_prefix);
  <b>if</b> (initial_amount &gt; 0) <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_deposit">LibraAccount::deposit</a>(
        fresh_address,
        <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>&lt;Token&gt;(initial_amount)
     );
}
</code></pre>



</details>
