
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`

The chain id distinguishes between different chains (e.g., testnet and the main Diem network).
One important role is to prevent transactions intended for one chain from being executed on another.
This code provides a container for storing a chain id and functions to initialize and get it.


-  [Resource `ChainId`](#0x1_ChainId_ChainId)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_ChainId_initialize)
-  [Function `get`](#0x1_ChainId_get)


<pre><code><b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`



<pre><code><b>struct</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ChainId_ECHAIN_ID"></a>

The <code><a href="ChainId.md#0x1_ChainId">ChainId</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_ECHAIN_ID">ECHAIN_ID</a>: u64 = 0;
</code></pre>



<a name="0x1_ChainId_initialize"></a>

## Function `initialize`

Publish the chain ID <code>id</code> of this Diem instance under the SystemAddresses address


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);
    <b>assert</b>!(!<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ChainId.md#0x1_ChainId_ECHAIN_ID">ECHAIN_ID</a>));
    <b>move_to</b>(account, <a href="ChainId.md#0x1_ChainId">ChainId</a> { id })
}
</code></pre>



</details>

<a name="0x1_ChainId_get"></a>

## Function `get`

Return the chain ID of this Diem instance


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8 <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>borrow_global</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(@CoreResources).id
}
</code></pre>



</details>
