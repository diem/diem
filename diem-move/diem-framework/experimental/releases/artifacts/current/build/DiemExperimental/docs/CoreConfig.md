
<a name="0x1_CoreConfig"></a>

# Module `0x1::CoreConfig`



-  [Resource `ConfigAccessPaths`](#0x1_CoreConfig_ConfigAccessPaths)
-  [Struct `NamedAccessPath`](#0x1_CoreConfig_NamedAccessPath)
-  [Function `initialize`](#0x1_CoreConfig_initialize)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_CoreConfig_ConfigAccessPaths"></a>

## Resource `ConfigAccessPaths`



<pre><code><b>struct</b> <a href="CoreConfig.md#0x1_CoreConfig_ConfigAccessPaths">ConfigAccessPaths</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>paths: vector&lt;<a href="CoreConfig.md#0x1_CoreConfig_NamedAccessPath">CoreConfig::NamedAccessPath</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_CoreConfig_NamedAccessPath"></a>

## Struct `NamedAccessPath`



<pre><code><b>struct</b> <a href="CoreConfig.md#0x1_CoreConfig_NamedAccessPath">NamedAccessPath</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>path: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_CoreConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="CoreConfig.md#0x1_CoreConfig_initialize">initialize</a>(root: &signer, version_config_path: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreConfig.md#0x1_CoreConfig_initialize">initialize</a>(root: &signer, version_config_path: vector&lt;u8&gt;) {
    // TODO: <b>assert</b> genesis
    <b>move_to</b>(
        root,
        <a href="CoreConfig.md#0x1_CoreConfig_ConfigAccessPaths">ConfigAccessPaths</a> {
            paths: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_singleton">Vector::singleton</a>(<a href="CoreConfig.md#0x1_CoreConfig_NamedAccessPath">NamedAccessPath</a> { name: b"DiemVersion", path: version_config_path })
        }
    );
}
</code></pre>



</details>
