
<a name="SCRIPT"></a>

# Script `rotate_base_url.move`

### Table of Contents

-  [Function `rotate_base_url`](#SCRIPT_rotate_base_url)



<a name="SCRIPT_rotate_base_url"></a>

## Function `rotate_base_url`

Rotate
<code>vasp_root_addr</code>'s base URL to
<code>new_url</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_base_url">rotate_base_url</a>(vasp: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_base_url">rotate_base_url</a>(vasp: &signer, new_url: vector&lt;u8&gt;) {
    <a href="../../modules/doc/VASP.md#0x1_VASP_rotate_base_url">VASP::rotate_base_url</a>(vasp, new_url)
}
</code></pre>



</details>
