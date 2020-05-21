
<a name="SCRIPT"></a>

# Script `remove_association_address.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(addr: address) {
    <a href="../../modules/doc/association.md#0x0_Association_remove_privilege">Association::remove_privilege</a>&lt;<a href="../../modules/doc/association.md#0x0_Association_T">Association::T</a>&gt;(addr)
}
</code></pre>



</details>
