
<a name="0x1_ChainMarker"></a>

# Module `0x1::ChainMarker`



-  [Resource `SingletonMarker`](#0x1_ChainMarker_SingletonMarker)
-  [Resource `ChainMarker`](#0x1_ChainMarker_ChainMarker)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_ChainMarker_initialize)


<pre><code><b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_ChainMarker_SingletonMarker"></a>

## Resource `SingletonMarker`



<pre><code><b>struct</b> <a href="ChainMarker.md#0x1_ChainMarker_SingletonMarker">SingletonMarker</a> has key
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

<a name="0x1_ChainMarker_ChainMarker"></a>

## Resource `ChainMarker`



<pre><code><b>struct</b> <a href="ChainMarker.md#0x1_ChainMarker">ChainMarker</a>&lt;Marker&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ChainMarker_ECHAINMARKER_ALREADY_PUBLISHED"></a>

Chain marker was already published at genesis


<pre><code><b>const</b> <a href="ChainMarker.md#0x1_ChainMarker_ECHAINMARKER_ALREADY_PUBLISHED">ECHAINMARKER_ALREADY_PUBLISHED</a>: u64 = 1;
</code></pre>



<a name="0x1_ChainMarker_EINVALID_CHAINMARKER"></a>

Invalid chain marker was supplied at genesis


<pre><code><b>const</b> <a href="ChainMarker.md#0x1_ChainMarker_EINVALID_CHAINMARKER">EINVALID_CHAINMARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_ChainMarker_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="ChainMarker.md#0x1_ChainMarker_initialize">initialize</a>&lt;Marker&gt;(chain_marker_signer: &signer, config_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainMarker.md#0x1_ChainMarker_initialize">initialize</a>&lt;Marker&gt;(
    chain_marker_signer: &signer,
    config_address: address,
)  {
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(chain_marker_signer);
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <b>assert</b>!(
        addr == @<a href="ChainMarker.md#0x1_ChainMarker">ChainMarker</a>,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ChainMarker.md#0x1_ChainMarker_EINVALID_CHAINMARKER">EINVALID_CHAINMARKER</a>)
    );
    <b>assert</b>!(!<b>exists</b>&lt;<a href="ChainMarker.md#0x1_ChainMarker_SingletonMarker">SingletonMarker</a>&gt;(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="ChainMarker.md#0x1_ChainMarker_ECHAINMARKER_ALREADY_PUBLISHED">ECHAINMARKER_ALREADY_PUBLISHED</a>));
    move_to(chain_marker_signer, <a href="ChainMarker.md#0x1_ChainMarker_SingletonMarker">SingletonMarker</a>{});
    move_to(chain_marker_signer, <a href="ChainMarker.md#0x1_ChainMarker">ChainMarker</a>&lt;Marker&gt; {
        config_address,
    })
}
</code></pre>



</details>
