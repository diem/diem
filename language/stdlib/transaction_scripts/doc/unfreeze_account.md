
<a name="SCRIPT"></a>

# Script `unfreeze_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(sliding_nonce: u64, to_unfreeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(sliding_nonce: u64, to_unfreeze_account: address) {
    <a href="../../modules/doc/sliding_nonce.md#0x0_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(sliding_nonce);
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_unfreeze_account">LibraAccount::unfreeze_account</a>(to_unfreeze_account);
}
</code></pre>



</details>
