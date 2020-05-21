
<a name="SCRIPT"></a>

# Script `register_preburner.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;() {
    <a href="../../modules/doc/libra.md#0x0_Libra_publish_preburn">Libra::publish_preburn</a>(<a href="../../modules/doc/libra.md#0x0_Libra_new_preburn">Libra::new_preburn</a>&lt;Token&gt;())
}
</code></pre>



</details>
