
<a name="SCRIPT"></a>

# Script `rotate_authentication_key.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, new_key: vector&lt;u8&gt;) {
  <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(account, new_key)
}
</code></pre>



</details>
