
<a name="SCRIPT"></a>

# Script `modify_publishing_option.move`

### Table of Contents

-  [Function `modify_publishing_option`](#SCRIPT_modify_publishing_option)



<a name="SCRIPT_modify_publishing_option"></a>

## Function `modify_publishing_option`

Modify publishing options. Takes the LCS bytes of a
<code>VMPublishingOption</code> object as input.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_modify_publishing_option">modify_publishing_option</a>(account: &signer, args: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_modify_publishing_option">modify_publishing_option</a>(account: &signer, args: vector&lt;u8&gt;) {
    <a href="../../modules/doc/LibraVMConfig.md#0x1_LibraVMConfig_set_publishing_option">LibraVMConfig::set_publishing_option</a>(account, args)
}
</code></pre>



</details>
