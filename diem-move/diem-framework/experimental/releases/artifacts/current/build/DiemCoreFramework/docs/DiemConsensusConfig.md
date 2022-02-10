
<a name="0x1_DiemConsensusConfig"></a>

# Module `0x1::DiemConsensusConfig`

Maintains the consensus config for the Diem blockchain. The config is stored in a
DiemConfig, and may be updated by Diem root.


-  [Resource `ConsensusConfigChainMarker`](#0x1_DiemConsensusConfig_ConsensusConfigChainMarker)
-  [Resource `DiemConsensusConfig`](#0x1_DiemConsensusConfig_DiemConsensusConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemConsensusConfig_initialize)
-  [Function `set`](#0x1_DiemConsensusConfig_set)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemConsensusConfig_ConsensusConfigChainMarker"></a>

## Resource `ConsensusConfigChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ConsensusConfigChainMarker">ConsensusConfigChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_DiemConsensusConfig_DiemConsensusConfig"></a>

## Resource `DiemConsensusConfig`



<pre><code><b>struct</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemConsensusConfig_ECHAIN_MARKER"></a>

Error with chain marker


<pre><code><b>const</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemConsensusConfig_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ECONFIG">ECONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemConsensusConfig_initialize"></a>

## Function `initialize`

Publishes the DiemConsensusConfig config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_initialize">initialize</a>&lt;T&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_initialize">initialize</a>&lt;T&gt;(account: &signer) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ConsensusConfigChainMarker">ConsensusConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ECONFIG">ECONFIG</a>)
    );
    <b>move_to</b>(account, <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ConsensusConfigChainMarker">ConsensusConfigChainMarker</a>&lt;T&gt;{});
    <b>move_to</b>(account, <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> { config: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>

<a name="0x1_DiemConsensusConfig_set"></a>

## Function `set`

Update the config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_set">set</a>&lt;T&gt;(config: vector&lt;u8&gt;, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_set">set</a>&lt;T&gt;(config: vector&lt;u8&gt;, _cap: &Cap&lt;T&gt;) <b>acquires</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ConsensusConfigChainMarker">ConsensusConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>));
    <b>let</b> config_ref = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;(@CoreResources).config;
    *config_ref = config;
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>
