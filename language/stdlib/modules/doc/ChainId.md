
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`



-  [Resource <code><a href="ChainId.md#0x1_ChainId">ChainId</a></code>](#0x1_ChainId_ChainId)
-  [Const <code><a href="ChainId.md#0x1_ChainId_ECHAIN_ID">ECHAIN_ID</a></code>](#0x1_ChainId_ECHAIN_ID)
-  [Function <code>initialize</code>](#0x1_ChainId_initialize)
-  [Function <code>get</code>](#0x1_ChainId_get)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`



<pre><code><b>resource</b> <b>struct</b> <a href="ChainId.md#0x1_ChainId">ChainId</a>
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

The <code><a href="ChainId.md#0x1_ChainId">ChainId</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_ECHAIN_ID">ECHAIN_ID</a>: u64 = 0;
</code></pre>



<a name="0x1_ChainId_initialize"></a>

## Function `initialize`

Publish the chain ID <code>id</code> of this Libra instance under the LibraRoot account


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(lr_account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(lr_account: &signer, id: u8) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ChainId.md#0x1_ChainId_ECHAIN_ID">ECHAIN_ID</a>));
    move_to(lr_account, <a href="ChainId.md#0x1_ChainId">ChainId</a> { id })
}
</code></pre>



</details>

<a name="0x1_ChainId_get"></a>

## Function `get`

Return the chain ID of this Libra instance


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8 <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    borrow_global&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).id
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>




<a name="0x1_ChainId_spec_get_chain_id"></a>


<pre><code><b>define</b> <a href="ChainId.md#0x1_ChainId_spec_get_chain_id">spec_get_chain_id</a>(): u8 {
    <b>global</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).id
}
</code></pre>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
