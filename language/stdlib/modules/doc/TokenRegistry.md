
<a name="0x1_TokenRegistry"></a>

# Module `0x1::TokenRegistry`



-  [Resource `IdCounter`](#0x1_TokenRegistry_IdCounter)
-  [Resource `TokenMetadata`](#0x1_TokenRegistry_TokenMetadata)
-  [Resource `TokenRegistryWithMintCapability`](#0x1_TokenRegistry_TokenRegistryWithMintCapability)
-  [Constants](#@Constants_0)
-  [Function `TOKEN_REGISTRY_COUNTER_ADDRESS`](#0x1_TokenRegistry_TOKEN_REGISTRY_COUNTER_ADDRESS)
-  [Function `initialize`](#0x1_TokenRegistry_initialize)
-  [Function `get_fresh_id`](#0x1_TokenRegistry_get_fresh_id)
-  [Function `register`](#0x1_TokenRegistry_register)
-  [Function `assert_is_registered_at`](#0x1_TokenRegistry_assert_is_registered_at)
-  [Function `is_transferable`](#0x1_TokenRegistry_is_transferable)
-  [Function `get_id`](#0x1_TokenRegistry_get_id)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_TokenRegistry_IdCounter"></a>

## Resource `IdCounter`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>count: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenRegistry_TokenMetadata"></a>

## Resource `TokenMetadata`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transferable: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenRegistry_TokenRegistryWithMintCapability"></a>

## Resource `TokenRegistryWithMintCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TokenRegistryWithMintCapability">TokenRegistryWithMintCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>maker_account: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_TokenRegistry_EID_COUNTER"></a>

A property expected of a <code><a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a></code> resource didn't hold


<pre><code><b>const</b> <a href="TokenRegistry.md#0x1_TokenRegistry_EID_COUNTER">EID_COUNTER</a>: u64 = 1;
</code></pre>



<a name="0x1_TokenRegistry_ETOKEN_REG"></a>

A property expected of a <code><a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a></code> resource didn't hold


<pre><code><b>const</b> <a href="TokenRegistry.md#0x1_TokenRegistry_ETOKEN_REG">ETOKEN_REG</a>: u64 = 2;
</code></pre>



<a name="0x1_TokenRegistry_TOKEN_REGISTRY_COUNTER_ADDRESS"></a>

## Function `TOKEN_REGISTRY_COUNTER_ADDRESS`



<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TOKEN_REGISTRY_COUNTER_ADDRESS">TOKEN_REGISTRY_COUNTER_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TOKEN_REGISTRY_COUNTER_ADDRESS">TOKEN_REGISTRY_COUNTER_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_initialize"></a>

## Function `initialize`

Initialization of the <code><a href="TokenRegistry.md#0x1_TokenRegistry">TokenRegistry</a></code> module; initializes
the counter of unique IDs


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_initialize">initialize</a>(
    config_account: &signer,
) {
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="TokenRegistry.md#0x1_TokenRegistry_EID_COUNTER">EID_COUNTER</a>)
    );
    move_to(config_account, <a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a> {count: 0});
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_get_fresh_id"></a>

## Function `get_fresh_id`

Returns a globally unique ID


<pre><code><b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_get_fresh_id">get_fresh_id</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_get_fresh_id">get_fresh_id</a>(): u64 <b>acquires</b> <a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a>{
    <b>let</b> addr = <a href="TokenRegistry.md#0x1_TokenRegistry_TOKEN_REGISTRY_COUNTER_ADDRESS">TOKEN_REGISTRY_COUNTER_ADDRESS</a>();
    <b>assert</b>(<b>exists</b>&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="TokenRegistry.md#0x1_TokenRegistry_EID_COUNTER">EID_COUNTER</a>));
    <b>let</b> id = borrow_global_mut&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a>&gt;(addr);
    id.count = id.count + 1;
    id.count
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_register"></a>

## Function `register`

Registers a new token, place its TokenMetadata on the signer
account and returns a designated mint capability


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_register">register</a>&lt;CoinType&gt;(maker_account: &signer, _t: &CoinType, transferable: bool): <a href="TokenRegistry.md#0x1_TokenRegistry_TokenRegistryWithMintCapability">TokenRegistry::TokenRegistryWithMintCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_register">register</a>&lt;CoinType&gt;(maker_account: &signer,
                            _t: &CoinType,
                            transferable: bool,
): <a href="TokenRegistry.md#0x1_TokenRegistry_TokenRegistryWithMintCapability">TokenRegistryWithMintCapability</a>&lt;CoinType&gt; <b>acquires</b> <a href="TokenRegistry.md#0x1_TokenRegistry_IdCounter">IdCounter</a> {
    <b>assert</b>(!<b>exists</b>&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(maker_account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="TokenRegistry.md#0x1_TokenRegistry_ETOKEN_REG">ETOKEN_REG</a>));
    // increments unique counter under <b>global</b> registry address
    <b>let</b> unique_id = <a href="TokenRegistry.md#0x1_TokenRegistry_get_fresh_id">get_fresh_id</a>();
    move_to&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;&gt;(
        maker_account,
        <a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a> { id: unique_id, transferable}
    );
    <b>let</b> address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(maker_account);
    <a href="TokenRegistry.md#0x1_TokenRegistry_TokenRegistryWithMintCapability">TokenRegistryWithMintCapability</a>&lt;CoinType&gt;{maker_account: address}
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_assert_is_registered_at"></a>

## Function `assert_is_registered_at`

Asserts that <code>CoinType</code> is a registered type at the given address


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_assert_is_registered_at">assert_is_registered_at</a>&lt;CoinType&gt;(registered_at: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_assert_is_registered_at">assert_is_registered_at</a>&lt;CoinType&gt; (registered_at: address){
    <b>assert</b>(<b>exists</b>&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;&gt;(registered_at), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="TokenRegistry.md#0x1_TokenRegistry_ETOKEN_REG">ETOKEN_REG</a>));
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_is_transferable"></a>

## Function `is_transferable`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is a transferable token.
Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_is_transferable">is_transferable</a>&lt;CoinType&gt;(registered_at: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_is_transferable">is_transferable</a>&lt;CoinType&gt;(registered_at: address): bool <b>acquires</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>{
    <a href="TokenRegistry.md#0x1_TokenRegistry_assert_is_registered_at">assert_is_registered_at</a>&lt;CoinType&gt;(registered_at);
    <b>let</b> metadata = borrow_global&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;&gt;(registered_at);
    metadata.transferable
}
</code></pre>



</details>

<a name="0x1_TokenRegistry_get_id"></a>

## Function `get_id`

Returns the global uniqe ID of <code>CoinType</code>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_get_id">get_id</a>&lt;CoinType&gt;(registered_at: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenRegistry.md#0x1_TokenRegistry_get_id">get_id</a>&lt;CoinType&gt;(registered_at: address): u64 <b>acquires</b> <a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>{
    <a href="TokenRegistry.md#0x1_TokenRegistry_assert_is_registered_at">assert_is_registered_at</a>&lt;CoinType&gt;(registered_at);
    <b>let</b> metadata = borrow_global&lt;<a href="TokenRegistry.md#0x1_TokenRegistry_TokenMetadata">TokenMetadata</a>&lt;CoinType&gt;&gt;(registered_at);
    metadata.id
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
