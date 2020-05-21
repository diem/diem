
<a name="SCRIPT"></a>

# Script `remove_child_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(child_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(child_address: address) {
    <a href="../../modules/doc/vasp.md#0x0_VASP_decertify_child_account">VASP::decertify_child_account</a>(child_address)
}
</code></pre>



</details>
