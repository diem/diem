
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`

### Table of Contents

-  [Resource `ChainId`](#0x1_ChainId_ChainId)
-  [Const `ECHAIN_ID`](#0x1_ChainId_ECHAIN_ID)
-  [Function `initialize`](#0x1_ChainId_initialize)
-  [Function `get`](#0x1_ChainId_get)
-  [Specification](#0x1_ChainId_Specification)



<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ChainId">ChainId</a>
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

<a name="0x1_ChainId_ECHAIN_ID"></a>

## Const `ECHAIN_ID`

The
<code><a href="#0x1_ChainId">ChainId</a></code> resource was not in the required state


<pre><code><b>const</b> ECHAIN_ID: u64 = 0;
</code></pre>



<a name="0x1_ChainId_initialize"></a>

## Function `initialize`

Publish the chain ID
<code>id</code> of this Libra instance under the LibraRoot account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_initialize">initialize</a>(lr_account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_initialize">initialize</a>(lr_account: &signer, id: u8) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(!exists&lt;<a href="#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ECHAIN_ID));
    move_to(lr_account, <a href="#0x1_ChainId">ChainId</a> { id })
}
</code></pre>



</details>

<a name="0x1_ChainId_get"></a>

## Function `get`

Return the chain ID of this Libra instance


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_get">get</a>(): u8 <b>acquires</b> <a href="#0x1_ChainId">ChainId</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    borrow_global&lt;<a href="#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).id
}
</code></pre>



</details>

<a name="0x1_ChainId_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; exists&lt;<a href="#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>
