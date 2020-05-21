
<a name="SCRIPT"></a>

# Script `remove_parent_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(parent_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(parent_address: address) {
    <a href="../../modules/doc/vasp.md#0x0_VASP_remove_parent_capability">VASP::remove_parent_capability</a>(parent_address)
}
</code></pre>



</details>
