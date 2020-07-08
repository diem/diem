
<a name="SCRIPT"></a>

# Script `rotate_base_url.move`

### Table of Contents

-  [Function `rotate_base_url`](#SCRIPT_rotate_base_url)



<a name="SCRIPT_rotate_base_url"></a>

## Function `rotate_base_url`

Rotate
<code>account</code>'s base URL to
<code>new_url</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_base_url">rotate_base_url</a>(account: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_base_url">rotate_base_url</a>(account: &signer, new_url: vector&lt;u8&gt;) {
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_rotate_base_url">DualAttestation::rotate_base_url</a>(account, new_url)
}
</code></pre>



</details>
