
<a name="0x1_ExperimentalVersion"></a>

# Module `0x1::ExperimentalVersion`

Maintains the version number for the blockchain.


-  [Struct `ExperimentalVersion`](#0x1_ExperimentalVersion_ExperimentalVersion)
-  [Function `initialize`](#0x1_ExperimentalVersion_initialize)
-  [Function `set`](#0x1_ExperimentalVersion_set)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="CoreVersion.md#0x1_CoreVersion">0x1::CoreVersion</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
</code></pre>



<a name="0x1_ExperimentalVersion_ExperimentalVersion"></a>

## Struct `ExperimentalVersion`



<pre><code><b>struct</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion">ExperimentalVersion</a> <b>has</b> drop
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

<a name="0x1_ExperimentalVersion_initialize"></a>

## Function `initialize`

Publishes the Version config.


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_initialize">initialize</a>(account: &signer, initial_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_initialize">initialize</a>(account: &signer, initial_version: u64) {
    <a href="CoreVersion.md#0x1_CoreVersion_initialize">CoreVersion::initialize</a>&lt;<a href="ExperimentalVersion.md#0x1_ExperimentalVersion">ExperimentalVersion</a>&gt;(account, initial_version);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_create">Capability::create</a>(account, &<a href="ExperimentalVersion.md#0x1_ExperimentalVersion">ExperimentalVersion</a>{});
}
</code></pre>



</details>

<a name="0x1_ExperimentalVersion_set"></a>

## Function `set`

Updates the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_set">set</a>(account: &signer, major: u64) {
    <a href="CoreVersion.md#0x1_CoreVersion_set">CoreVersion::set</a>&lt;<a href="ExperimentalVersion.md#0x1_ExperimentalVersion">ExperimentalVersion</a>&gt;(
        major,
        &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_acquire">Capability::acquire</a>(account, &<a href="ExperimentalVersion.md#0x1_ExperimentalVersion">ExperimentalVersion</a>{})
    );
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>(account);
}
</code></pre>



</details>
