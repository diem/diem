
<a name="SCRIPT"></a>

# Script `update_libra_version.move`

### Table of Contents

-  [Function `update_libra_version`](#SCRIPT_update_libra_version)



<a name="SCRIPT_update_libra_version"></a>

## Function `update_libra_version`

Update Libra version.
<code>sliding_nonce</code> is a unique nonce for operation, see sliding_nonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion_set">LibraVersion::set</a>(account, major)
}
</code></pre>



</details>
