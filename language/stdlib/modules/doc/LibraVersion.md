
<a name="0x1_LibraVersion"></a>

# Module `0x1::LibraVersion`

### Table of Contents

-  [Struct `LibraVersion`](#0x1_LibraVersion_LibraVersion)
-  [Const `EINVALID_MAJOR_VERSION_NUMBER`](#0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER)
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

<a name="0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER"></a>

## Const `EINVALID_MAJOR_VERSION_NUMBER`

Tried to set an invalid major version for the VM. Major versions must be strictly increasing


<pre><code><b>const</b> EINVALID_MAJOR_VERSION_NUMBER: u64 = 0;
</code></pre>



<a name="0x1_LibraVersion_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVersion_initialize">initialize</a>(
    lr_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();

    // TODO: is this restricted <b>to</b> be called from libra root?
    // <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(account);

    <b>let</b> old_config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;();

    <b>assert</b>(
        old_config.major &lt; major,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EINVALID_MAJOR_VERSION_NUMBER)
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



After genesis, version is published.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;();
</code></pre>


The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [B20].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where exists&lt;<a href="#0x1_LibraVersion">LibraVersion</a>&gt;(addr):
    addr == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>
