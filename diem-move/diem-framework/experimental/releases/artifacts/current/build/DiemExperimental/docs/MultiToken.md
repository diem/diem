
<a name="0x1_MultiToken"></a>

# Module `0x1::MultiToken`



-  [Resource `TokenData`](#0x1_MultiToken_TokenData)
-  [Struct `TokenDataWrapper`](#0x1_MultiToken_TokenDataWrapper)
-  [Resource `Token`](#0x1_MultiToken_Token)
-  [Struct `MintEvent`](#0x1_MultiToken_MintEvent)
-  [Resource `Admin`](#0x1_MultiToken_Admin)
-  [Resource `TokenDataCollection`](#0x1_MultiToken_TokenDataCollection)
-  [Constants](#@Constants_0)
-  [Function `id`](#0x1_MultiToken_id)
-  [Function `balance`](#0x1_MultiToken_balance)
-  [Function `metadata`](#0x1_MultiToken_metadata)
-  [Function `supply`](#0x1_MultiToken_supply)
-  [Function `extract_token`](#0x1_MultiToken_extract_token)
-  [Function `restore_token`](#0x1_MultiToken_restore_token)
-  [Function `index_of_token`](#0x1_MultiToken_index_of_token)
-  [Function `join`](#0x1_MultiToken_join)
-  [Function `split`](#0x1_MultiToken_split)
-  [Function `initialize_multi_token`](#0x1_MultiToken_initialize_multi_token)
-  [Function `create`](#0x1_MultiToken_create)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID">0x1::GUID</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_MultiToken_TokenData"></a>

## Resource `TokenData`

Struct representing data of a specific token with token_id,
stored under the creator's address inside TokenInfoCollection.
For each token_id, there is only one MultiTokenData.


<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_TokenData">TokenData</a>&lt;TokenType: store&gt; has store, key
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
</dl>


</details>

<a name="0x1_MultiToken_TokenDataWrapper"></a>

## Struct `TokenDataWrapper`

A hot potato wrapper for the token's metadata. Since this wrapper has no <code>key</code> or <code>store</code>
ability, it can't be stored in global storage. This wrapper can be safely passed outside
of this module because we know it will have to come back to this module, where
it will be unpacked.


<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType: store&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>origin: address</code>
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
</dl>


</details>

<a name="0x1_MultiToken_Token"></a>

## Resource `Token`

Struct representing a semi-fungible or non-fungible token (depending on the supply).
There can be multiple tokens with the same id (unless supply is 1). Each token's
corresponding token metadata is stored inside a MultiTokenData inside TokenDataCollection
under the creator's address.


<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType: store&gt; has store, key
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

<a name="0x1_MultiToken_MintEvent"></a>

## Struct `MintEvent`



<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_MintEvent">MintEvent</a> has <b>copy</b>, drop, store
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
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_MultiToken_Admin"></a>

## Resource `Admin`



<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_Admin">Admin</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="MultiToken.md#0x1_MultiToken_MintEvent">MultiToken::MintEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_MultiToken_TokenDataCollection"></a>

## Resource `TokenDataCollection`



<pre><code><b>struct</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>tokens: vector&lt;<a href="MultiToken.md#0x1_MultiToken_TokenData">MultiToken::TokenData</a>&lt;TokenType&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_MultiToken_MAX_U64"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_MAX_U64">MAX_U64</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_MultiToken_ADMIN"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_ADMIN">ADMIN</a>: address = a550c18;
</code></pre>



<a name="0x1_MultiToken_EAMOUNT_EXCEEDS_TOKEN_BALANCE"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_EAMOUNT_EXCEEDS_TOKEN_BALANCE">EAMOUNT_EXCEEDS_TOKEN_BALANCE</a>: u64 = 3;
</code></pre>



<a name="0x1_MultiToken_EINDEX_EXCEEDS_LENGTH"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_EINDEX_EXCEEDS_LENGTH">EINDEX_EXCEEDS_LENGTH</a>: u64 = 5;
</code></pre>



<a name="0x1_MultiToken_ENOT_ADMIN"></a>

Function can only be called by the admin address


<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_ENOT_ADMIN">ENOT_ADMIN</a>: u64 = 0;
</code></pre>



<a name="0x1_MultiToken_ETOKEN_BALANCE_OVERFLOWS"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_ETOKEN_BALANCE_OVERFLOWS">ETOKEN_BALANCE_OVERFLOWS</a>: u64 = 2;
</code></pre>



<a name="0x1_MultiToken_ETOKEN_EXTRACTED"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_ETOKEN_EXTRACTED">ETOKEN_EXTRACTED</a>: u64 = 4;
</code></pre>



<a name="0x1_MultiToken_ETOKEN_PRESENT"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_ETOKEN_PRESENT">ETOKEN_PRESENT</a>: u64 = 6;
</code></pre>



<a name="0x1_MultiToken_EWRONG_TOKEN_ID"></a>



<pre><code><b>const</b> <a href="MultiToken.md#0x1_MultiToken_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>: u64 = 1;
</code></pre>



<a name="0x1_MultiToken_id"></a>

## Function `id`

Returns the id of given token


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_id">id</a>&lt;TokenType: store&gt;(token: &<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_id">id</a>&lt;TokenType: store&gt;(token: &<a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a> {
    *&token.id
}
</code></pre>



</details>

<a name="0x1_MultiToken_balance"></a>

## Function `balance`

Returns the balance of given token


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_balance">balance</a>&lt;TokenType: store&gt;(token: &<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_balance">balance</a>&lt;TokenType: store&gt;(token: &<a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;): u64 {
    token.balance
}
</code></pre>



</details>

<a name="0x1_MultiToken_metadata"></a>

## Function `metadata`



<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_metadata">metadata</a>&lt;TokenType: store&gt;(wrapper: &<a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">MultiToken::TokenDataWrapper</a>&lt;TokenType&gt;): &TokenType
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_metadata">metadata</a>&lt;TokenType: store&gt;(wrapper: &<a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt;): &TokenType {
    &wrapper.metadata
}
</code></pre>



</details>

<a name="0x1_MultiToken_supply"></a>

## Function `supply`

Returns the supply of tokens with <code>id</code> on the chain.


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_supply">supply</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_supply">supply</a>&lt;TokenType: store&gt;(id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): u64 <b>acquires</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> owner_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(id);
    <b>let</b> tokens = &<b>mut</b> borrow_global_mut&lt;<a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(owner_addr).tokens;
    <b>let</b> index_opt = <a href="MultiToken.md#0x1_MultiToken_index_of_token">index_of_token</a>&lt;TokenType&gt;(tokens, id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="MultiToken.md#0x1_MultiToken_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>));
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tokens, index).supply
}
</code></pre>



</details>

<a name="0x1_MultiToken_extract_token"></a>

## Function `extract_token`

Extract the MultiToken data of the given token into a hot potato wrapper.


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_extract_token">extract_token</a>&lt;TokenType: store&gt;(nft: &<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;): <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">MultiToken::TokenDataWrapper</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_extract_token">extract_token</a>&lt;TokenType: store&gt;(nft: &<a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;): <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt; <b>acquires</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> owner_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id_creator_address">GUID::id_creator_address</a>(&nft.id);
    <b>let</b> tokens = &<b>mut</b> borrow_global_mut&lt;<a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(owner_addr).tokens;
    <b>let</b> index_opt = <a href="MultiToken.md#0x1_MultiToken_index_of_token">index_of_token</a>&lt;TokenType&gt;(tokens, &nft.id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="MultiToken.md#0x1_MultiToken_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>));
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
    <b>let</b> item_opt = &<b>mut</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tokens, index).metadata;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(item_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="MultiToken.md#0x1_MultiToken_ETOKEN_EXTRACTED">ETOKEN_EXTRACTED</a>));
    <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a> { origin: owner_addr, index, metadata: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(item_opt) }
}
</code></pre>



</details>

<a name="0x1_MultiToken_restore_token"></a>

## Function `restore_token`

Restore the token in the wrapper back into global storage under original address.


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_restore_token">restore_token</a>&lt;TokenType: store&gt;(wrapper: <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">MultiToken::TokenDataWrapper</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_restore_token">restore_token</a>&lt;TokenType: store&gt;(wrapper: <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a>&lt;TokenType&gt;) <b>acquires</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> <a href="MultiToken.md#0x1_MultiToken_TokenDataWrapper">TokenDataWrapper</a> { origin, index, metadata } = wrapper;
    <b>let</b> tokens = &<b>mut</b> borrow_global_mut&lt;<a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(origin).tokens;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(tokens) &gt; index, <a href="MultiToken.md#0x1_MultiToken_EINDEX_EXCEEDS_LENGTH">EINDEX_EXCEEDS_LENGTH</a>);
    <b>let</b> item_opt = &<b>mut</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tokens, index).metadata;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_none">Option::is_none</a>(item_opt), <a href="MultiToken.md#0x1_MultiToken_ETOKEN_PRESENT">ETOKEN_PRESENT</a>);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_fill">Option::fill</a>(item_opt, metadata);
}
</code></pre>



</details>

<a name="0x1_MultiToken_index_of_token"></a>

## Function `index_of_token`

Finds the index of token with the given id in the gallery.


<pre><code><b>fun</b> <a href="MultiToken.md#0x1_MultiToken_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;<a href="MultiToken.md#0x1_MultiToken_TokenData">MultiToken::TokenData</a>&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MultiToken.md#0x1_MultiToken_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;<a href="MultiToken.md#0x1_MultiToken_TokenData">TokenData</a>&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;u64&gt; {
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

<a name="0x1_MultiToken_join"></a>

## Function `join`

Join two multi tokens and return a multi token with the combined value of the two.


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_join">join</a>&lt;TokenType: store&gt;(token: &<b>mut</b> <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;, other: <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_join">join</a>&lt;TokenType: store&gt;(token: &<b>mut</b> <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;, other: <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="MultiToken.md#0x1_MultiToken_Token">Token</a> { id, balance } = other;
    <b>assert</b>!(*&token.id == id, <a href="MultiToken.md#0x1_MultiToken_EWRONG_TOKEN_ID">EWRONG_TOKEN_ID</a>);
    <b>assert</b>!(<a href="MultiToken.md#0x1_MultiToken_MAX_U64">MAX_U64</a> - token.balance &gt;= balance, <a href="MultiToken.md#0x1_MultiToken_ETOKEN_BALANCE_OVERFLOWS">ETOKEN_BALANCE_OVERFLOWS</a>);
    token.balance = token.balance + balance
}
</code></pre>



</details>

<a name="0x1_MultiToken_split"></a>

## Function `split`

Split the token into two tokens, one with balance <code>amount</code> and the other one with balance


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_split">split</a>&lt;TokenType: store&gt;(token: <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;, amount: u64): (<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;, <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_split">split</a>&lt;TokenType: store&gt;(token: <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;, amount: u64): (<a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;, <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt;) {
    <b>assert</b>!(token.balance &gt;= amount, <a href="MultiToken.md#0x1_MultiToken_EAMOUNT_EXCEEDS_TOKEN_BALANCE">EAMOUNT_EXCEEDS_TOKEN_BALANCE</a>);
    token.balance = token.balance - amount;
    <b>let</b> id = *&token.id;
    (token,
    <a href="MultiToken.md#0x1_MultiToken_Token">Token</a> {
        id,
        balance: amount
    } )
}
</code></pre>



</details>

<a name="0x1_MultiToken_initialize_multi_token"></a>

## Function `initialize_multi_token`

Initialize this module, to be called in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_initialize_multi_token">initialize_multi_token</a>(account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_initialize_multi_token">initialize_multi_token</a>(account: signer) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&account) == <a href="MultiToken.md#0x1_MultiToken_ADMIN">ADMIN</a>, <a href="MultiToken.md#0x1_MultiToken_ENOT_ADMIN">ENOT_ADMIN</a>);
    move_to(&account, <a href="MultiToken.md#0x1_MultiToken_Admin">Admin</a> {
        mint_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="MultiToken.md#0x1_MultiToken_MintEvent">MintEvent</a>&gt;(&account),
    })
}
</code></pre>



</details>

<a name="0x1_MultiToken_create"></a>

## Function `create`

Create a<code> <a href="MultiToken.md#0x1_MultiToken_TokenData">TokenData</a>&lt;TokenType&gt;</code> that wraps <code>metadata</code> and with balance of <code>amount</code>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_create">create</a>&lt;TokenType: store&gt;(account: &signer, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64): <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiToken.md#0x1_MultiToken_create">create</a>&lt;TokenType: store&gt;(
    account: &signer, metadata: TokenType, content_uri: vector&lt;u8&gt;, amount: u64
): <a href="MultiToken.md#0x1_MultiToken_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="MultiToken.md#0x1_MultiToken_Admin">Admin</a>, <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a> {
    <b>let</b> guid = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_create">GUID::create</a>(account);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> borrow_global_mut&lt;<a href="MultiToken.md#0x1_MultiToken_Admin">Admin</a>&gt;(<a href="MultiToken.md#0x1_MultiToken_ADMIN">ADMIN</a>).mint_events,
        <a href="MultiToken.md#0x1_MultiToken_MintEvent">MintEvent</a> {
            id: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(&guid),
            creator: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
            content_uri: <b>copy</b> content_uri,
            amount,
        }
    );
    <b>let</b> id = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(&guid);
    <b>if</b> (!<b>exists</b>&lt;<a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        move_to(account, <a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a> { tokens: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="MultiToken.md#0x1_MultiToken_TokenData">TokenData</a>&lt;TokenType&gt;&gt;() });
    };
    <b>let</b> token_data_collection = &<b>mut</b> borrow_global_mut&lt;<a href="MultiToken.md#0x1_MultiToken_TokenDataCollection">TokenDataCollection</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).tokens;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(
        token_data_collection,
        <a href="MultiToken.md#0x1_MultiToken_TokenData">TokenData</a> { metadata: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(metadata), token_id: guid, content_uri, supply: amount }
    );
    <a href="MultiToken.md#0x1_MultiToken_Token">Token</a> { id, balance: amount }
}
</code></pre>



</details>
