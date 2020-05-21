
<a name="SCRIPT"></a>

# Script `apply_for_root_vasp.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, travel_rule_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    travel_rule_public_key: vector&lt;u8&gt;
) {
    <a href="../../modules/doc/vasp.md#0x0_VASP_apply_for_vasp_root_credential">VASP::apply_for_vasp_root_credential</a>(human_name, base_url, travel_rule_public_key);
}
</code></pre>



</details>
