
<a name="SCRIPT"></a>

# Script `mint_lbr_to_address.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(payee: address, auth_key_prefix: vector&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(payee: address, auth_key_prefix: vector&lt;u8&gt;, amount: u64) {
  <b>if</b> (!<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_exists">LibraAccount::exists</a>(payee)) {
      <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;<a href="../../modules/doc/lbr.md#0x0_LBR_T">LBR::T</a>&gt;(payee, auth_key_prefix);
  };
  <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_mint_lbr_to_address">LibraAccount::mint_lbr_to_address</a>(payee, amount);
}
</code></pre>



</details>
