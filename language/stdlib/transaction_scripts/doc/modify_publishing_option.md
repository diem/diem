
<a name="SCRIPT"></a>

# Script `modify_publishing_option.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, args: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, args: vector&lt;u8&gt;) {
    <a href="../../modules/doc/LibraVMConfig.md#0x1_LibraVMConfig_set_publishing_option">LibraVMConfig::set_publishing_option</a>(account, args)
}
</code></pre>



</details>
