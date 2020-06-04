
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_nonce.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(sliding_nonce: u64, new_key: vector&lt;u8&gt;) {
  <a href="../../modules/doc/sliding_nonce.md#0x0_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(sliding_nonce);
  <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(new_key)
}
</code></pre>



</details>
