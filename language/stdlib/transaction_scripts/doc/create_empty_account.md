
<a name="SCRIPT"></a>

# Script `create_empty_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(fresh_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(fresh_address: address, auth_key_prefix: vector&lt;u8&gt;) {
  <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_create_account">LibraAccount::create_account</a>&lt;Token&gt;(fresh_address, auth_key_prefix);
}
</code></pre>



</details>
