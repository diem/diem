
<a name="SCRIPT"></a>

# Script `rotate_compliance_public_key.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(vasp: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(vasp: &signer, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/VASP.md#0x1_VASP_rotate_compliance_public_key">VASP::rotate_compliance_public_key</a>(vasp, new_key)
}
</code></pre>



</details>
