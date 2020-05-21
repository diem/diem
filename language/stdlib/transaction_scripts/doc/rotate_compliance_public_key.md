
<a name="SCRIPT"></a>

# Script `rotate_compliance_public_key.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(root_vasp_address: address, new_compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(root_vasp_address: address, new_compliance_public_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/vasp.md#0x0_VASP_rotate_compliance_public_key">VASP::rotate_compliance_public_key</a>(root_vasp_address, new_compliance_public_key)
}
</code></pre>



</details>
