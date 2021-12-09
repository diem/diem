
<a name="0x1_NFT"></a>

# Module `0x1::NFT`



-  [Resource `TokenData`](#0x1_NFT_TokenData)
-  [Struct `TokenDataWrapper`](#0x1_NFT_TokenDataWrapper)
-  [Resource `Token`](#0x1_NFT_Token)
-  [Struct `MintEvent`](#0x1_NFT_MintEvent)
-  [Struct `TransferEvent`](#0x1_NFT_TransferEvent)
-  [Resource `Admin`](#0x1_NFT_Admin)
-  [Resource `TokenDataCollection`](#0x1_NFT_TokenDataCollection)
-  [Resource `CreationDelegation`](#0x1_NFT_CreationDelegation)
-  [Constants](#@Constants_0)
-  [Function `id`](#0x1_NFT_id)
-  [Function `balance`](#0x1_NFT_balance)
-  [Function `metadata`](#0x1_NFT_metadata)
-  [Function `parent`](#0x1_NFT_parent)
-  [Function `royalty_rate`](#0x1_NFT_royalty_rate)
-  [Function `supply`](#0x1_NFT_supply)
-  [Function `extract_token`](#0x1_NFT_extract_token)
-  [Function `restore_token`](#0x1_NFT_restore_token)
-  [Function `index_of_token`](#0x1_NFT_index_of_token)
-  [Function `join`](#0x1_NFT_join)
-  [Function `split`](#0x1_NFT_split)
-  [Function `nft_initialize`](#0x1_NFT_nft_initialize)
-  [Function `create_for`](#0x1_NFT_create_for)
-  [Function `create`](#0x1_NFT_create)
-  [Function `create_impl`](#0x1_NFT_create_impl)
-  [Function `publish_token_data_collection`](#0x1_NFT_publish_token_data_collection)
-  [Function `allow_creation_delegation`](#0x1_NFT_allow_creation_delegation)
-  [Function `emit_transfer_event`](#0x1_NFT_emit_transfer_event)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">0x1::FixedPoint32</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID">0x1::GUID</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NFT_TokenData"></a>

## Resource `TokenData`

Struct representing data of a specific token with token_id,
stored under the creator's address inside TokenDataCollection.
For each token_id, there is only one TokenData.


<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_TokenData">TokenData</a>&lt;TokenType: store&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>metadata: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>token_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_GUID">GUID::GUID</a></code>
</dt>
<dd>
 Identifier for the token.
</dd>
<dt>
<code>content_uri: vector&lt;u8&gt;</code>
</dt>
<dd>
 Pointer to where the content and metadata is stored.
</dd>
<dt>
<code>supply: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;</code>
</dt>
<dd>
 Parent NFT id
</dd>
<dt>
<code>royalty_rate: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>&gt;</code>
</dt>
<dd>
 Percentage of sales going back to the creator
</dd>
</dl>


</details>

<a name="0x1_NFT_TokenDataWrapper"></a>

## Struct `TokenDataWrapper`

A hot potato wrapper for the token's metadata. Since this wrapper has no <code>key</code> or <code>store</code>
ability, it can't be stored in global storage. This wrapper can be safely passed outside
of this module because we know it will have to come back to this module, where
it will be unpacked.


<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType: store&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>origin: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>index: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata: TokenType</code>
</dt>
<dd>

</dd>
<dt>
<code>parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_Token"></a>

## Resource `Token`

Struct representing a semi-fungible or non-fungible token (depending on the supply).
There can be multiple tokens with the same id (unless supply is 1). Each token's
corresponding token metadata is stored inside a TokenData inside TokenDataCollection
under the creator's address.


<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType: store&gt; <b>has</b> store, key
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
<code>balance: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_MintEvent"></a>

## Struct `MintEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a> <b>has</b> <b>copy</b>, drop, store
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
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>content_uri: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_TransferEvent"></a>

## Struct `TransferEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_TransferEvent">TransferEvent</a> <b>has</b> <b>copy</b>, drop, store
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
<code>from: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code><b>to</b>: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_Admin"></a>

## Resource `Admin`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFT_MintEvent">NFT::MintEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>transfer_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFT_TransferEvent">NFT::TransferEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_TokenDataCollection"></a>

## Resource `TokenDataCollection`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType: store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>tokens: vector&lt;<a href="NFT.md#0x1_NFT_TokenData">NFT::TokenData</a>&lt;TokenType&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFT_CreationDelegation"></a>

## Resource `CreationDelegation`

Indicates that a user allows creation delegation for a given TokenType


<pre><code><b>struct</b> <a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>&lt;TokenType: store&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>guid_capability: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_CreateCapability">GUID::CreateCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_NFT_MAX_U64"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_MAX_U64">MAX_U64</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_NFT_ADMIN"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>: <b>address</b> = a550c18;
</code></pre>



<a name="0x1_NFT_EAMOUNT_EXCEEDS_TOKEN_BALANCE"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_EAMOUNT_EXCEEDS_TOKEN_BALANCE">EAMOUNT_EXCEEDS_TOKEN_BALANCE</a>: u64 = 3;
</code></pre>



<a name="0x1_NFT_ECREATION_DELEGATION_NOT_ALLOWED"></a>

Creation delegation for a given token type is not allowed.


<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ECREATION_DELEGATION_NOT_ALLOWED">ECREATION_DELEGATION_NOT_ALLOWED</a>: u64 = 9;
</code></pre>



<a name="0x1_NFT_EINDEX_EXCEEDS_LENGTH"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_EINDEX_EXCEEDS_LENGTH">EINDEX_EXCEEDS_LENGTH</a>: u64 = 5;
</code></pre>



<a name="0x1_NFT_ENOT_ADMIN"></a>

Function can only be called by the admin address


<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ENOT_ADMIN">ENOT_ADMIN</a>: u64 = 0;
</code></pre>



<a name="0x1_NFT_EPARENT_NOT_SAME_ACCOUNT"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_EPARENT_NOT_SAME_ACCOUNT">EPARENT_NOT_SAME_ACCOUNT</a>: u64 = 7;
</code></pre>



<a name="0x1_NFT_ETOKEN_BALANCE_OVERFLOWS"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ETOKEN_BALANCE_OVERFLOWS">ETOKEN_BALANCE_OVERFLOWS</a>: u64 = 2;
</code></pre>



<a name="0x1_NFT_ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED">ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED</a>: u64 = 8;
</code></pre>



<a name="0x1_NFT_ETOKEN_EXTRACTED"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ETOKEN_EXTRACTED">ETOKEN_EXTRACTED</a>: u64 = 4;
</code></pre>



<a name="0x1_NFT_ETOKEN_PRESENT"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_ETOKEN_PRESENT">ETOKEN_PRESENT</a>: u64 = 6;
</code></pre>



<a name="0x1_NFT_EWRONG_TOKEN_ID"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFT_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>: u64 = 1;
</code></pre>



<a name="0x1_NFT_id"></a>

## Function `id`

Returns the id of given token


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_id">id</a>&lt;TokenType: store&gt;(token: &<a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_id">id</a>&lt;TokenType: store&gt;(token: &<a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a> {
    *&token.id
}
</code></pre>



</details>

<a name="0x1_NFT_balance"></a>

## Function `balance`

Returns the balance of given token


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_balance">balance</a>&lt;TokenType: store&gt;(token: &<a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_balance">balance</a>&lt;TokenType: store&gt;(token: &<a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;): u64 {
    token.balance
}
</code></pre>



</details>

<a name="0x1_NFT_metadata"></a>

## Function `metadata`



<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_metadata">metadata</a>&lt;TokenType: store&gt;(wrapper: &<a href="NFT.md#0x1_NFT_TokenDataWrapper">NFT::TokenDataWrapper</a>&lt;TokenType&gt;): &TokenType
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_metadata">metadata</a>&lt;TokenType: store&gt;(wrapper: &<a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt;): &TokenType {
    &wrapper.metadata
}
</code></pre>



</details>

<a name="0x1_NFT_parent"></a>

## Function `parent`

Returns ID of collection associated with token


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_parent">parent</a>&lt;TokenType: store&gt;(wrapper: &<a href="NFT.md#0x1_NFT_TokenDataWrapper">NFT::TokenDataWrapper</a>&lt;TokenType&gt;): &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_parent">parent</a>&lt;TokenType: store&gt;(wrapper: &<a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt;): &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt; {
    &wrapper.parent_id
}
</code></pre>



</details>

<a name="0x1_NFT_royalty_rate"></a>

## Function `royalty_rate`

Returns the royalty rate of token with <code>id</code> on the chain.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_royalty_rate">royalty_rate</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_royalty_rate">royalty_rate</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> owner_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(id);
    <b>let</b> tokens = &<b>borrow_global</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(owner_addr).tokens;
    <b>let</b> index_opt = <a href="NFT.md#0x1_NFT_index_of_token">index_of_token</a>&lt;TokenType&gt;(tokens, id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="NFT.md#0x1_NFT_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>));
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
    *&<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tokens, index).royalty_rate
}
</code></pre>



</details>

<a name="0x1_NFT_supply"></a>

## Function `supply`

Returns the supply of tokens with <code>id</code> on the chain.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_supply">supply</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_supply">supply</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): u64 <b>acquires</b> <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> owner_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(id);
    <b>let</b> tokens = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(owner_addr).tokens;
    <b>let</b> index_opt = <a href="NFT.md#0x1_NFT_index_of_token">index_of_token</a>&lt;TokenType&gt;(tokens, id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="NFT.md#0x1_NFT_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>));
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tokens, index).supply
}
</code></pre>



</details>

<a name="0x1_NFT_extract_token"></a>

## Function `extract_token`

Extract the Token data of the given token into a hot potato wrapper.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_extract_token">extract_token</a>&lt;TokenType: store&gt;(nft: &<a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;): <a href="NFT.md#0x1_NFT_TokenDataWrapper">NFT::TokenDataWrapper</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_extract_token">extract_token</a>&lt;TokenType: store&gt;(nft: &<a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;): <a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> owner_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(&nft.id);
    <b>let</b> tokens = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(owner_addr).tokens;
    <b>let</b> index_opt = <a href="NFT.md#0x1_NFT_index_of_token">index_of_token</a>&lt;TokenType&gt;(tokens, &nft.id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="NFT.md#0x1_NFT_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>));
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
    <b>let</b> item_opt = &<b>mut</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tokens, index).metadata;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(item_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="NFT.md#0x1_NFT_ETOKEN_EXTRACTED">ETOKEN_EXTRACTED</a>));
    <b>let</b> metadata = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(item_opt);
    <b>let</b> parent_opt = &<b>mut</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tokens, index).parent_id;
    <a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a> { origin: owner_addr, index, metadata, parent_id: *parent_opt }
}
</code></pre>



</details>

<a name="0x1_NFT_restore_token"></a>

## Function `restore_token`

Restore the token in the wrapper back into global storage under original address.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_restore_token">restore_token</a>&lt;TokenType: store&gt;(wrapper: <a href="NFT.md#0x1_NFT_TokenDataWrapper">NFT::TokenDataWrapper</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_restore_token">restore_token</a>&lt;TokenType: store&gt;(wrapper: <a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt;) <b>acquires</b> <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> <a href="NFT.md#0x1_NFT_TokenDataWrapper">TokenDataWrapper</a> { origin, index, metadata, parent_id: _ } = wrapper;
    <b>let</b> tokens = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(origin).tokens;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(tokens) &gt; index, <a href="NFT.md#0x1_NFT_EINDEX_EXCEEDS_LENGTH">EINDEX_EXCEEDS_LENGTH</a>);
    <b>let</b> item_opt = &<b>mut</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tokens, index).metadata;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_none">Option::is_none</a>(item_opt), <a href="NFT.md#0x1_NFT_ETOKEN_PRESENT">ETOKEN_PRESENT</a>);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_fill">Option::fill</a>(item_opt, metadata);
}
</code></pre>



</details>

<a name="0x1_NFT_index_of_token"></a>

## Function `index_of_token`

Finds the index of token with the given id in the gallery.


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFT_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;<a href="NFT.md#0x1_NFT_TokenData">NFT::TokenData</a>&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFT_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;<a href="NFT.md#0x1_NFT_TokenData">TokenData</a>&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(gallery);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_eq_id">GUID::eq_id</a>(&<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(gallery, i).token_id, id)) {
            <b>return</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x1_NFT_join"></a>

## Function `join`

Join two tokens and return a new token with the combined value of the two.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_join">join</a>&lt;TokenType: store&gt;(token: &<b>mut</b> <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;, other: <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_join">join</a>&lt;TokenType: store&gt;(token: &<b>mut</b> <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;, other: <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="NFT.md#0x1_NFT_Token">Token</a> { id, balance } = other;
    <b>assert</b>!(*&token.id == id, <a href="NFT.md#0x1_NFT_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>);
    <b>assert</b>!(<a href="NFT.md#0x1_NFT_MAX_U64">MAX_U64</a> - token.balance &gt;= balance, <a href="NFT.md#0x1_NFT_ETOKEN_BALANCE_OVERFLOWS">ETOKEN_BALANCE_OVERFLOWS</a>);
    token.balance = token.balance + balance
}
</code></pre>



</details>

<a name="0x1_NFT_split"></a>

## Function `split`

Split the token into two tokens, one with balance <code>amount</code> and the other one with balance


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_split">split</a>&lt;TokenType: store&gt;(token: <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;, amount: u64): (<a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;, <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_split">split</a>&lt;TokenType: store&gt;(token: <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;, amount: u64): (<a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;, <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt;) {
    <b>assert</b>!(token.balance &gt;= amount, <a href="NFT.md#0x1_NFT_EAMOUNT_EXCEEDS_TOKEN_BALANCE">EAMOUNT_EXCEEDS_TOKEN_BALANCE</a>);
    token.balance = token.balance - amount;
    <b>let</b> id = *&token.id;
    (token,
        <a href="NFT.md#0x1_NFT_Token">Token</a> {
            id,
            balance: amount
        } )
}
</code></pre>



</details>

<a name="0x1_NFT_nft_initialize"></a>

## Function `nft_initialize`

Initialize this module


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFT_nft_initialize">nft_initialize</a>(account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFT_nft_initialize">nft_initialize</a>(account: signer) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&account) == <a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>, <a href="NFT.md#0x1_NFT_ENOT_ADMIN">ENOT_ADMIN</a>);
    <b>move_to</b>(&account, <a href="NFT.md#0x1_NFT_Admin">Admin</a> {
        mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a>&gt;(&account),
        transfer_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFT_TransferEvent">TransferEvent</a>&gt;(&account),
    })
}
</code></pre>



</details>

<a name="0x1_NFT_create_for"></a>

## Function `create_for`

Create an NFT on behalf of the given user, in case a user explicitly approved this delegation for the given
NFT type.
Only the entity, which can create an object of <code>TokenType</code>, will be able to call this function.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create_for">create_for</a>&lt;TokenType: store&gt;(creator: <b>address</b>, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64, parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;): <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create_for">create_for</a>&lt;TokenType: store&gt;(
    creator: <b>address</b>, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64, parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;
): <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>, <a href="NFT.md#0x1_NFT_Admin">Admin</a>, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>assert</b>! (<b>exists</b>&lt;<a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>&lt;TokenType&gt;&gt;(creator), <a href="NFT.md#0x1_NFT_ECREATION_DELEGATION_NOT_ALLOWED">ECREATION_DELEGATION_NOT_ALLOWED</a>);
    <b>let</b> guid_creation_cap = &<b>borrow_global</b>&lt;<a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>&lt;TokenType&gt;&gt;(creator).guid_capability;
    <b>let</b> guid = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_create_with_capability">GUID::create_with_capability</a>(creator, guid_creation_cap);
    <a href="NFT.md#0x1_NFT_create_impl">create_impl</a>&lt;TokenType&gt;(
        creator,
        guid,
        metadata,
        content_uri,
        amount,
        parent_id,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>()
    )
}
</code></pre>



</details>

<a name="0x1_NFT_create"></a>

## Function `create`

Create a<code> <a href="NFT.md#0x1_NFT_TokenData">TokenData</a>&lt;TokenType&gt;</code> that wraps <code>metadata</code> and with balance of <code>amount</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create">create</a>&lt;TokenType: store&gt;(account: &signer, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64, parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;, royalty_rate: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>&gt;): <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_create">create</a>&lt;TokenType: store&gt;(
    account: &signer, metadata: TokenType, content_uri: vector&lt;u8&gt;,
    amount: u64, parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;, royalty_rate: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>&gt;
): <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a>, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> guid = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_create">GUID::create</a>(account);
    <b>if</b> (!<b>exists</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <b>move_to</b>(account, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> { tokens: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="NFT.md#0x1_NFT_TokenData">TokenData</a>&lt;TokenType&gt;&gt;() });
    };
    <a href="NFT.md#0x1_NFT_create_impl">create_impl</a>&lt;TokenType&gt;(
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
        guid,
        metadata,
        content_uri,
        amount,
        parent_id,
        royalty_rate,
    )
}
</code></pre>



</details>

<a name="0x1_NFT_create_impl"></a>

## Function `create_impl`



<pre><code><b>fun</b> <a href="NFT.md#0x1_NFT_create_impl">create_impl</a>&lt;TokenType: store&gt;(addr: <b>address</b>, guid: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_GUID">GUID::GUID</a>, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64, parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;, royalty_rate: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>&gt;): <a href="NFT.md#0x1_NFT_Token">NFT::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFT_create_impl">create_impl</a>&lt;TokenType: store&gt;(
    addr: <b>address</b>,
    guid: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_GUID">GUID::GUID</a>,
    metadata: TokenType,
    content_uri: vector&lt;u8&gt;,
    amount: u64,
    parent_id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>&gt;,
    royalty_rate: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>&gt;,
): <a href="NFT.md#0x1_NFT_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a>, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> {
    // If there is a parent, ensure it <b>has</b> the same creator
    // TODO: Do we just say the owner <b>has</b> the ability instead?
    <b>if</b> (<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&parent_id)) {
        <b>let</b> parent_id = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(&<b>mut</b> parent_id);
        <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_creator_address">GUID::creator_address</a>(&guid) == <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(parent_id), <a href="NFT.md#0x1_NFT_EPARENT_NOT_SAME_ACCOUNT">EPARENT_NOT_SAME_ACCOUNT</a>);
    };
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_Admin">Admin</a>&gt;(<a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>).mint_events,
        <a href="NFT.md#0x1_NFT_MintEvent">MintEvent</a> {
            id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(&guid),
            creator: addr,
            content_uri: <b>copy</b> content_uri,
            amount,
        }
    );
    <b>let</b> id = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(&guid);
    <b>let</b> token_data_collection = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(addr).tokens;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(
        token_data_collection,
        <a href="NFT.md#0x1_NFT_TokenData">TokenData</a> {
            metadata: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(metadata),
            token_id: guid,
            content_uri,
            supply: amount,
            parent_id,
            royalty_rate
        }
    );
    <a href="NFT.md#0x1_NFT_Token">Token</a> { id, balance: amount }
}
</code></pre>



</details>

<a name="0x1_NFT_publish_token_data_collection"></a>

## Function `publish_token_data_collection`



<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_publish_token_data_collection">publish_token_data_collection</a>&lt;TokenType: store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_publish_token_data_collection">publish_token_data_collection</a>&lt;TokenType: store&gt;(account: &signer) {
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="NFT.md#0x1_NFT_ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED">ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED</a>
    );
    <b>move_to</b>(account, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt; { tokens: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>

<a name="0x1_NFT_allow_creation_delegation"></a>

## Function `allow_creation_delegation`

Allow creation delegation for a given TokenType (the entity, which can generate a metadata of a given TokenType
is going to be allowed to create an NFT on behalf of the user).
This is useful in case a user is using a 3rd party app, which can create NFTs on their behalf.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_allow_creation_delegation">allow_creation_delegation</a>&lt;TokenType: store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_allow_creation_delegation">allow_creation_delegation</a>&lt;TokenType: store&gt;(account: &signer) {
    <b>if</b> (!<b>exists</b>&lt;<a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <b>move_to</b>(account, <a href="NFT.md#0x1_NFT_CreationDelegation">CreationDelegation</a>&lt;TokenType&gt; { guid_capability: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_gen_create_capability">GUID::gen_create_capability</a>(account) });
        // In order <b>to</b> support creation delegation, prepare the token data collection ahead of time.
        <b>if</b> (!<b>exists</b>&lt;<a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
            <b>move_to</b>(account, <a href="NFT.md#0x1_NFT_TokenDataCollection">TokenDataCollection</a> { tokens: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="NFT.md#0x1_NFT_TokenData">TokenData</a>&lt;TokenType&gt;&gt;() });
        };
    };
}
</code></pre>



</details>

<a name="0x1_NFT_emit_transfer_event"></a>

## Function `emit_transfer_event`



<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_emit_transfer_event">emit_transfer_event</a>(guid: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>, account: &signer, <b>to</b>: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFT_emit_transfer_event">emit_transfer_event</a>(
    guid: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>,
    account: &signer,
    <b>to</b>: <b>address</b>,
    amount: u64,
) <b>acquires</b> <a href="NFT.md#0x1_NFT_Admin">Admin</a> {
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NFT.md#0x1_NFT_Admin">Admin</a>&gt;(<a href="NFT.md#0x1_NFT_ADMIN">ADMIN</a>).transfer_events,
        <a href="NFT.md#0x1_NFT_TransferEvent">TransferEvent</a> {
            id: *guid,
            from: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
            <b>to</b>: <b>to</b>,
            amount: amount,
        }
    );
}
</code></pre>



</details>
