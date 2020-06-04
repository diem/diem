
<a name="SCRIPT"></a>

# Script `rotate_consensus_pubkey.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a> (new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/ValidatorConfig.md#0x0_ValidatorConfig_set_consensus_pubkey">ValidatorConfig::set_consensus_pubkey</a>(Transaction::sender(), new_key);
    <a href="../../modules/doc/LibraSystem.md#0x0_LibraSystem_update_and_reconfigure">LibraSystem::update_and_reconfigure</a>();
}
</code></pre>



</details>
