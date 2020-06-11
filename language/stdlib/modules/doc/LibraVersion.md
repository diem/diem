
<a name="0x0_LibraVersion"></a>

# Module `0x0::LibraVersion`

### Table of Contents

-  [Struct `LibraVersion`](#0x0_LibraVersion_LibraVersion)
-  [Function `initialize`](#0x0_LibraVersion_initialize)
-  [Function `set`](#0x0_LibraVersion_set)



<a name="0x0_LibraVersion_LibraVersion"></a>

## Struct `LibraVersion`



<pre><code><b>struct</b> <a href="#0x0_LibraVersion">LibraVersion</a>
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_initialize">initialize</a>(account: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>(), 1);

    <a href="LibraConfig.md#0x0_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>&lt;<a href="#0x0_LibraVersion">LibraVersion</a>&gt;(
        account,
        <a href="#0x0_LibraVersion">LibraVersion</a> { major: 1 },
    );
}
</code></pre>



</details>

<a name="0x0_LibraVersion_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraVersion_set">set</a>(account: &signer, major: u64) {
    <b>let</b> old_config = <a href="LibraConfig.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_LibraVersion">LibraVersion</a>&gt;();

    Transaction::assert(
        old_config.major &lt; major,
        25
    );

    <a href="LibraConfig.md#0x0_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x0_LibraVersion">LibraVersion</a>&gt;(
        account,
        <a href="#0x0_LibraVersion">LibraVersion</a> { major }
    );
}
</code></pre>



</details>
