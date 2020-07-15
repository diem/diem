
<a name="0x1_LibraVersion"></a>

# Module `0x1::LibraVersion`

### Table of Contents

-  [Struct `LibraVersion`](#0x1_LibraVersion_LibraVersion)
-  [Function `initialize`](#0x1_LibraVersion_initialize)
-  [Function `set`](#0x1_LibraVersion_set)
-  [Specification](#0x1_LibraVersion_Specification)



<a name="0x1_LibraVersion_LibraVersion"></a>

## Struct `LibraVersion`



<pre><code><b>struct</b> <a href="#0x1_LibraVersion">LibraVersion</a>
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

<a name="0x1_LibraVersion_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_initialize">initialize</a>(
    lr_account: &signer,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;(
        lr_account,
        <a href="#0x1_LibraVersion">LibraVersion</a> { major: 1 },
    );
}
</code></pre>



</details>

<a name="0x1_LibraVersion_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_set">set</a>(account: &signer, major: u64) {
    <b>let</b> old_config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;();

    <b>assert</b>(
        old_config.major &lt; major,
        EINVALID_MAJOR_VERSION_NUMBER
    );

    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;(
        account,
        <a href="#0x1_LibraVersion">LibraVersion</a> { major }
    );
}
</code></pre>



</details>

<a name="0x1_LibraVersion_Specification"></a>

## Specification


The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [B20].


<pre><code><b>invariant</b> forall addr: address where exists&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;(addr):
    addr == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
</code></pre>
