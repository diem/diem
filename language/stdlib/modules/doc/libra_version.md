
<a name="0x0_LibraVersion"></a>

# Module `0x0::LibraVersion`

### Table of Contents

-  [Struct `T`](#0x0_LibraVersion_T)
-  [Function `initialize`](#0x0_LibraVersion_initialize)
-  [Function `set`](#0x0_LibraVersion_set)



<a name="0x0_LibraVersion_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_LibraVersion_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>major: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraVersion_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_initialize">initialize</a>() {
    Transaction::assert(Transaction::sender() == <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(), 1);

    <a href="libra_configs.md#0x0_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>&lt;<a href="#0x0_LibraVersion_T">Self::T</a>&gt;(
        <a href="#0x0_LibraVersion_T">T</a> { major: 1 },
    );
}
</code></pre>



</details>

<a name="0x0_LibraVersion_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_set">set</a>(major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_set">set</a>(major: u64) {
    <b>let</b> old_config = <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_LibraVersion_T">Self::T</a>&gt;();

    Transaction::assert(
        old_config.major &lt; major,
        25
    );

    <a href="libra_configs.md#0x0_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x0_LibraVersion_T">Self::T</a>&gt;(
        <a href="#0x0_LibraVersion_T">T</a> { major }
    );
}
</code></pre>



</details>
