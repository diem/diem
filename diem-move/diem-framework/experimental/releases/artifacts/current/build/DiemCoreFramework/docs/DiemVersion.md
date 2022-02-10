
<a name="0x1_DiemVersion"></a>

# Module `0x1::DiemVersion`

Maintains the version number for the blockchain.


-  [Resource `VersionChainMarker`](#0x1_DiemVersion_VersionChainMarker)
-  [Resource `DiemVersion`](#0x1_DiemVersion_DiemVersion)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemVersion_initialize)
-  [Function `set`](#0x1_DiemVersion_set)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_DiemVersion_VersionChainMarker"></a>

## Resource `VersionChainMarker`

Marker to be stored under 0x1 during genesis


<pre><code><b>struct</b> <a href="DiemVersion.md#0x1_DiemVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DiemVersion_DiemVersion"></a>

## Resource `DiemVersion`



<pre><code><b>struct</b> <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a> <b>has</b> <b>copy</b>, drop, store, key
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemVersion_ECHAIN_MARKER"></a>

Error with chain marker


<pre><code><b>const</b> <a href="DiemVersion.md#0x1_DiemVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemVersion_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER"></a>

Tried to set an invalid major version for the VM. Major versions must be strictly increasing


<pre><code><b>const</b> <a href="DiemVersion.md#0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemVersion_initialize"></a>

## Function `initialize`

Publishes the Version config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();

    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>)
    );

    <b>move_to</b>(
        account,
        <a href="DiemVersion.md#0x1_DiemVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt; {},
    );
    <b>move_to</b>(
        account,
        <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a> { major: initial_version },
    );
}
</code></pre>



</details>

<a name="0x1_DiemVersion_set"></a>

## Function `set`

Updates the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_set">set</a>&lt;T&gt;(major: u64, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_set">set</a>&lt;T&gt;(major: u64, _cap: &Cap&lt;T&gt;) <b>acquires</b> <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>));
    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>));
    <b>let</b> old_major = *&<b>borrow_global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&gt;(@CoreResources).major;

    <b>assert</b>!(
        old_major &lt; major,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemVersion.md#0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>)
    );

    <b>let</b> config = <b>borrow_global_mut</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&gt;(@CoreResources);
    config.major = major;

    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>
