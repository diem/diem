
<a name="0x1_NFT"></a>

# Module `0x1::NFT`



-  [Resource `NFT`](#0x1_NFT_NFT)
-  [Struct `MintEvent`](#0x1_NFT_MintEvent)
-  [Struct `TransferEvent`](#0x1_NFT_TransferEvent)
-  [Resource `Admin`](#0x1_NFT_Admin)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_NFT_initialize)
-  [Function `create`](#0x1_NFT_create)
-  [Function `publish`](#0x1_NFT_publish)
-  [Function `remove`](#0x1_NFT_remove)
-  [Function `id`](#0x1_NFT_id)
-  [Function `creator`](#0x1_NFT_creator)
-  [Function `token`](#0x1_NFT_token)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID">0x1::GUID</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_NFT_NFT"></a>

## Resource `NFT`

A non-fungible token of a specific <code>Type</code>, created by <code>id.addr</code>.
Anyone can create a <code><a href="NFT.md#0x1_NFT">NFT</a></code>. The access control policy for creating an <code><a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;</code> should be defined in the
logic for creating <code>Type</code>. For example, if only Michelangelo should be able to  create <code><a href="NFT.md#0x1_NFT">NFT</a>&lt;MikePainting&gt;</code>,
the <code>MikePainting</code> type should only be creatable by Michelangelo's address.


<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT">NFT</a>&lt;Type: drop, store&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token: Type</code>
</dt>
<dd>
 A struct to enable type-specific fields that will be different for each Token.
 For example, <code><a href="NFT.md#0x1_NFT">NFT</a>&lt;Painting&gt;</code> with
 <code><b>struct</b> Painting { name: vector&lt;u84, painter: vector&lt;u8&gt;, year: u64, ... }</code>,
 Or, <code><a href="NFT.md#0x1_NFT">NFT</a>&lt;DigitalPirateInGameItem&gt; { item_type: u8, item_power: u8, ... }</code>. Mutable.
</dd>
<dt>
<code>token_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_GUID">GUID::GUID</a></code>
</dt>
<dd>
 A globally unique identifier, which includes the address of the NFT
 creator (who may or may not be the same as the content creator). Immutable.
</dd>
<dt>
<code>content_uri: vector&lt;u8&gt;</code>
</dt>
<dd>
 pointer to where the content and metadata is stored. Could be a DiemID domain, IPFS, Dropbox url, etc. Immutable.
</dd>
<dt>
<code>content_hash: vector&lt;u8&gt;</code>
</dt>
<dd>
 cryptographic hash of the NFT's contents (e.g., hash of the bytes corresponding to a video)
</dd>
</dl>


</details>

<a name="0x1_NFT_MintEvent"></a>

## Struct `MintEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a>&lt;Type&gt; has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>creator: address</code>
</dt>
<dd>

</dd>
<dt>
<code>content_uri: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_TransferEvent"></a>

## Struct `TransferEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_TransferEvent">TransferEvent</a>&lt;Type&gt; has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>from: address</code>
</dt>
<dd>

</dd>
<dt>
<code><b>to</b>: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_Admin"></a>

## Resource `Admin`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a>&lt;Type&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFT_MintEvent">NFT::MintEvent</a>&lt;Type&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_NFT_ADMIN"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>: address = a550c18;
</code></pre>



<a name="0x1_NFT_ENOT_ADMIN"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ENOT_ADMIN">ENOT_ADMIN</a>: u64 = 0;
</code></pre>



<a name="0x1_NFT_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_initialize">initialize</a>&lt;Type: drop, store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_initialize">initialize</a>&lt;Type: store + drop&gt;(account: &signer) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>, <a href="NFT.md#0x1_NFT_ENOT_ADMIN">ENOT_ADMIN</a>);
    move_to(account, <a href="NFT.md#0x1_NFT_Admin">Admin</a> { mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a>&lt;Type&gt;&gt;(account) })
}
</code></pre>



</details>

<a name="0x1_NFT_create"></a>

## Function `create`

Create a<code> <a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;</code> that wraps <code>token</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create">create</a>&lt;Type: drop, store&gt;(account: &signer, token: Type, content_uri: vector&lt;u8&gt;): <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create">create</a>&lt;Type: store + drop&gt;(
    account: &signer, token: Type, content_uri: vector&lt;u8&gt;
): <a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a> {
    <b>let</b> creator = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> token_id = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_create">GUID::create</a>(account);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> borrow_global_mut&lt;<a href="NFT.md#0x1_NFT_Admin">Admin</a>&lt;Type&gt;&gt;(<a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>).mint_events,
        <a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a> {
            id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(&token_id),
            creator,
            content_uri: <b>copy</b> content_uri
        }
    );
    // TODO: take this <b>as</b> input
    <b>let</b> content_hash = x"";
    <a href="NFT.md#0x1_NFT">NFT</a> { token, token_id, content_uri, content_hash }
}
</code></pre>



</details>

<a name="0x1_NFT_publish"></a>

## Function `publish`

Publish the non-fungible token <code>nft</code> under <code>account</code>.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_publish">publish</a>&lt;Type: drop, store&gt;(account: &signer, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_publish">publish</a>&lt;Type: store + drop&gt;(account: &signer, nft: <a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;) {
    move_to(account, nft)
}
</code></pre>



</details>

<a name="0x1_NFT_remove"></a>

## Function `remove`

Remove the <code><a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;</code> under <code>account</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_remove">remove</a>&lt;Type: drop, store&gt;(account: &signer): <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_remove">remove</a>&lt;Type: store + drop&gt;(account: &signer): <a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT">NFT</a> {
    move_from&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x1_NFT_id"></a>

## Function `id`

Return the globally unique identifier of <code>nft</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_id">id</a>&lt;Type: drop, store&gt;(nft: &<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;): &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_GUID">GUID::GUID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_id">id</a>&lt;Type: store + drop&gt;(nft: &<a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;): &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID">GUID</a> {
    &nft.token_id
}
</code></pre>



</details>

<a name="0x1_NFT_creator"></a>

## Function `creator`

Return the creator of this NFT


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_creator">creator</a>&lt;Type: drop, store&gt;(nft: &<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_creator">creator</a>&lt;Type: store + drop&gt;(nft: &<a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;): address {
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_creator_address">GUID::creator_address</a>(<a href="NFT.md#0x1_NFT_id">id</a>&lt;Type&gt;(nft))
}
</code></pre>



</details>

<a name="0x1_NFT_token"></a>

## Function `token`

View the underlying token of a NFT


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_token">token</a>&lt;Type: drop, store&gt;(nft: &<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;Type&gt;): &Type
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_token">token</a>&lt;Type: store + drop&gt;(nft: &<a href="NFT.md#0x1_NFT">NFT</a>&lt;Type&gt;): &Type {
    &nft.token
}
</code></pre>



</details>
