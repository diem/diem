
<a name="SCRIPT"></a>

# Script `modify_publishing_option.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(args: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(args: vector&lt;u8&gt;) {
    <a href="../../modules/doc/libra_vm_config.md#0x0_LibraVMConfig_set_publishing_option">LibraVMConfig::set_publishing_option</a>(args)
}
</code></pre>



</details>
