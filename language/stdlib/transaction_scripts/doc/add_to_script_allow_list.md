
<a name="SCRIPT"></a>

# Script `add_to_script_allow_list.move`

### Table of Contents

-  [Function `add_to_script_allow_list`](#SCRIPT_add_to_script_allow_list)



<a name="SCRIPT_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`

Append the
<code>hash</code> to script hashes list allowed to be executed by the network.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64,) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">LibraTransactionPublishingOption::add_to_script_allow_list</a>(lr_account, hash)
}
</code></pre>



</details>
