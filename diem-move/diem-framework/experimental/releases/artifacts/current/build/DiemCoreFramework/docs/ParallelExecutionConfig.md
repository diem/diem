
<a name="0x1_ParallelExecutionConfig"></a>

# Module `0x1::ParallelExecutionConfig`

This module defines structs and methods to initialize VM configurations,
including different costs of running the VM.


-  [Resource `ParallelExecutionConfigChainMarker`](#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker)
-  [Resource `ParallelExecutionConfig`](#0x1_ParallelExecutionConfig_ParallelExecutionConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize_parallel_execution`](#0x1_ParallelExecutionConfig_initialize_parallel_execution)
-  [Function `enable_parallel_execution_with_config`](#0x1_ParallelExecutionConfig_enable_parallel_execution_with_config)
-  [Function `disable_parallel_execution`](#0x1_ParallelExecutionConfig_disable_parallel_execution)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker"></a>

## Resource `ParallelExecutionConfigChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker">ParallelExecutionConfigChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_ParallelExecutionConfig_ParallelExecutionConfig"></a>

## Resource `ParallelExecutionConfig`

The struct to hold the read/write set analysis result for the whole Diem Framework.


<pre><code><b>struct</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>read_write_analysis_result: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>
 Serialized analysis result for the Diem Framework.
 If this payload is not None, DiemVM will use this config to execute transactions in parallel.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ParallelExecutionConfig_ECHAIN_MARKER"></a>

Error with chain marker


<pre><code><b>const</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_ParallelExecutionConfig_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECONFIG">ECONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_ParallelExecutionConfig_initialize_parallel_execution"></a>

## Function `initialize_parallel_execution`

Enable parallel execution functionality of DiemVM by setting the read_write_set analysis result.


<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_initialize_parallel_execution">initialize_parallel_execution</a>&lt;T&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_initialize_parallel_execution">initialize_parallel_execution</a>&lt;T&gt;(
    account: &signer,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker">ParallelExecutionConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a>&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECONFIG">ECONFIG</a>)
    );

    <b>move_to</b>(account, <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker">ParallelExecutionConfigChainMarker</a>&lt;T&gt;{});

    <b>move_to</b>(
        account,
        <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a> {
            read_write_analysis_result: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>(),
        },
    );
}
</code></pre>



</details>

<a name="0x1_ParallelExecutionConfig_enable_parallel_execution_with_config"></a>

## Function `enable_parallel_execution_with_config`



<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_enable_parallel_execution_with_config">enable_parallel_execution_with_config</a>&lt;T&gt;(read_write_inference_result: vector&lt;u8&gt;, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_enable_parallel_execution_with_config">enable_parallel_execution_with_config</a>&lt;T&gt;(
    read_write_inference_result: vector&lt;u8&gt;,
    _cap: &Cap&lt;T&gt;
) <b>acquires</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker">ParallelExecutionConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );
    <b>let</b> result_ref = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a>&gt;(@CoreResources).read_write_analysis_result;
    *result_ref = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(read_write_inference_result);
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>

<a name="0x1_ParallelExecutionConfig_disable_parallel_execution"></a>

## Function `disable_parallel_execution`



<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_disable_parallel_execution">disable_parallel_execution</a>&lt;T&gt;(_cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_disable_parallel_execution">disable_parallel_execution</a>&lt;T&gt;(
    _cap: &Cap&lt;T&gt;
) <b>acquires</b> <a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ParallelExecutionConfigChainMarker">ParallelExecutionConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );
    <b>let</b> result_ref = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="ParallelExecutionConfig.md#0x1_ParallelExecutionConfig">ParallelExecutionConfig</a>&gt;(@CoreResources).read_write_analysis_result;
    *result_ref = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>();
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>
