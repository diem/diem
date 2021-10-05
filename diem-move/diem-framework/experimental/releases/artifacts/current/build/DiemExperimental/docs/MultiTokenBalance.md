
<a name="0x1_MultiTokenBalance"></a>

# Module `0x1::MultiTokenBalance`



-  [Resource `TokenBalance`](#0x1_MultiTokenBalance_TokenBalance)
-  [Constants](#@Constants_0)
-  [Function `add_to_gallery`](#0x1_MultiTokenBalance_add_to_gallery)
-  [Function `remove_from_gallery`](#0x1_MultiTokenBalance_remove_from_gallery)
-  [Function `index_of_token`](#0x1_MultiTokenBalance_index_of_token)
-  [Function `has_token`](#0x1_MultiTokenBalance_has_token)
-  [Function `get_token_balance`](#0x1_MultiTokenBalance_get_token_balance)
-  [Function `transfer_multi_token_between_galleries`](#0x1_MultiTokenBalance_transfer_multi_token_between_galleries)
-  [Function `publish_balance`](#0x1_MultiTokenBalance_publish_balance)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID">0x1::GUID</a>;
<b>use</b> <a href="MultiToken.md#0x1_MultiToken">0x1::MultiToken</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_MultiTokenBalance_TokenBalance"></a>

## Resource `TokenBalance`

Balance holding tokens of <code>TokenType</code> as well as information of approved operators.


<pre><code><b>struct</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>gallery: vector&lt;<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;&gt;</code>
</dt>
<dd>
 Gallery full of multi tokens owned by this balance
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_MultiTokenBalance_MAX_U64"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_MultiTokenBalance_EALREADY_IS_OPERATOR"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EALREADY_IS_OPERATOR">EALREADY_IS_OPERATOR</a>: u64 = 4;
</code></pre>



<a name="0x1_MultiTokenBalance_EBALANCE_ALREADY_PUBLISHED"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EBALANCE_ALREADY_PUBLISHED">EBALANCE_ALREADY_PUBLISHED</a>: u64 = 2;
</code></pre>



<a name="0x1_MultiTokenBalance_EBALANCE_NOT_PUBLISHED"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EBALANCE_NOT_PUBLISHED">EBALANCE_NOT_PUBLISHED</a>: u64 = 1;
</code></pre>



<a name="0x1_MultiTokenBalance_EID_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EID_NOT_FOUND">EID_NOT_FOUND</a>: u64 = 0;
</code></pre>



<a name="0x1_MultiTokenBalance_EINVALID_AMOUNT_OF_TRANSFER"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EINVALID_AMOUNT_OF_TRANSFER">EINVALID_AMOUNT_OF_TRANSFER</a>: u64 = 3;
</code></pre>



<a name="0x1_MultiTokenBalance_EINVALID_APPROVAL_TARGET"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EINVALID_APPROVAL_TARGET">EINVALID_APPROVAL_TARGET</a>: u64 = 6;
</code></pre>



<a name="0x1_MultiTokenBalance_ENOT_OPERATOR"></a>



<pre><code><b>const</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_ENOT_OPERATOR">ENOT_OPERATOR</a>: u64 = 5;
</code></pre>



<a name="0x1_MultiTokenBalance_add_to_gallery"></a>

## Function `add_to_gallery`

Add a token to the owner's gallery. If there is already a token of the same id in the
gallery, we combine it with the new one and make a token of greater value.


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_add_to_gallery">add_to_gallery</a>&lt;TokenType: store&gt;(owner: address, token: <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_add_to_gallery">add_to_gallery</a>&lt;TokenType: store&gt;(owner: address, token: Token&lt;TokenType&gt;)
<b>acquires</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner), <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EBALANCE_NOT_PUBLISHED">EBALANCE_NOT_PUBLISHED</a>);
    <b>let</b> id = <a href="MultiToken.md#0x1_MultiToken_id">MultiToken::id</a>&lt;TokenType&gt;(&token);
    <b>if</b> (<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_has_token">has_token</a>&lt;TokenType&gt;(owner, &id)) {
        // If `owner` already has a token <b>with</b> the same id, remove it from the gallery
        // and join it <b>with</b> the new token.
        <b>let</b> original_token = <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_remove_from_gallery">remove_from_gallery</a>&lt;TokenType&gt;(owner, &id);
        <a href="MultiToken.md#0x1_MultiToken_join">MultiToken::join</a>&lt;TokenType&gt;(&<b>mut</b> token, original_token);
    };
    <b>let</b> gallery = &<b>mut</b> borrow_global_mut&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner).gallery;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(gallery, token)
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_remove_from_gallery"></a>

## Function `remove_from_gallery`

Remove a token of certain id from the owner's gallery and return it.


<pre><code><b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_remove_from_gallery">remove_from_gallery</a>&lt;TokenType: store&gt;(owner: address, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_remove_from_gallery">remove_from_gallery</a>&lt;TokenType: store&gt;(owner: address, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): Token&lt;TokenType&gt;
<b>acquires</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner), <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EBALANCE_NOT_PUBLISHED">EBALANCE_NOT_PUBLISHED</a>);
    <b>let</b> gallery = &<b>mut</b> borrow_global_mut&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner).gallery;
    <b>let</b> index_opt = <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_index_of_token">index_of_token</a>&lt;TokenType&gt;(gallery, id);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&index_opt), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EID_NOT_FOUND">EID_NOT_FOUND</a>));
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_remove">Vector::remove</a>(gallery, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt))
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_index_of_token"></a>

## Function `index_of_token`

Finds the index of token with the given id in the gallery.


<pre><code><b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;<a href="MultiToken.md#0x1_MultiToken_Token">MultiToken::Token</a>&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_index_of_token">index_of_token</a>&lt;TokenType: store&gt;(gallery: &vector&lt;Token&lt;TokenType&gt;&gt;, id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(gallery);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="MultiToken.md#0x1_MultiToken_id">MultiToken::id</a>&lt;TokenType&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(gallery, i)) == *id) {
            <b>return</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_has_token"></a>

## Function `has_token`

Returns whether the owner has a token with given id.


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_has_token">has_token</a>&lt;TokenType: store&gt;(owner: address, token_id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_has_token">has_token</a>&lt;TokenType: store&gt;(owner: address, token_id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): bool <b>acquires</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a> {
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_index_of_token">index_of_token</a>(&borrow_global&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner).gallery, token_id))
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_get_token_balance"></a>

## Function `get_token_balance`



<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_get_token_balance">get_token_balance</a>&lt;TokenType: store&gt;(owner: address, token_id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_get_token_balance">get_token_balance</a>&lt;TokenType: store&gt;(owner: address, token_id: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_ID">GUID::ID</a>
): u64 <b>acquires</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a> {
    <b>let</b> gallery = &borrow_global&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(owner).gallery;
    <b>let</b> index_opt = <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_index_of_token">index_of_token</a>&lt;TokenType&gt;(gallery, token_id);

    <b>if</b> (<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_none">Option::is_none</a>(&index_opt)) {
        0
    } <b>else</b> {
        <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> index_opt);
        <a href="MultiToken.md#0x1_MultiToken_balance">MultiToken::balance</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(gallery, index))
    }
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_transfer_multi_token_between_galleries"></a>

## Function `transfer_multi_token_between_galleries`

Transfer <code>amount</code> of token with id <code><a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_id">GUID::id</a>(creator, creation_num)</code> from <code>owner</code>'s
balance to <code><b>to</b></code>'s balance. This operation has to be done by either the owner or an
approved operator of the owner.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_transfer_multi_token_between_galleries">transfer_multi_token_between_galleries</a>&lt;TokenType: store&gt;(account: signer, <b>to</b>: address, amount: u64, creator: address, creation_num: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_transfer_multi_token_between_galleries">transfer_multi_token_between_galleries</a>&lt;TokenType: store&gt;(
    account: signer,
    <b>to</b>: address,
    amount: u64,
    creator: address,
    creation_num: u64
) <b>acquires</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a> {
    <b>let</b> owner = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&account);

    <b>assert</b>!(amount &gt; 0, <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EINVALID_AMOUNT_OF_TRANSFER">EINVALID_AMOUNT_OF_TRANSFER</a>);

    // Remove <a href="NFT.md#0x1_NFT">NFT</a> from `owner`'s gallery
    <b>let</b> id = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID_create_id">GUID::create_id</a>(creator, creation_num);
    <b>let</b> token = <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_remove_from_gallery">remove_from_gallery</a>&lt;TokenType&gt;(owner, &id);

    <b>assert</b>!(amount &lt;= <a href="MultiToken.md#0x1_MultiToken_balance">MultiToken::balance</a>(&token), <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EINVALID_AMOUNT_OF_TRANSFER">EINVALID_AMOUNT_OF_TRANSFER</a>);

    <b>if</b> (amount == <a href="MultiToken.md#0x1_MultiToken_balance">MultiToken::balance</a>(&token)) {
        // Owner does not have any token left, so add token <b>to</b> `<b>to</b>`'s gallery.
        <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_add_to_gallery">add_to_gallery</a>&lt;TokenType&gt;(<b>to</b>, token);
    } <b>else</b> {
        // Split owner's token into two
        <b>let</b> (owner_token, to_token) = <a href="MultiToken.md#0x1_MultiToken_split">MultiToken::split</a>&lt;TokenType&gt;(token, amount);

        // Add tokens <b>to</b> owner's gallery.
        <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_add_to_gallery">add_to_gallery</a>&lt;TokenType&gt;(owner, owner_token);

        // Add tokens <b>to</b> `<b>to</b>`'s gallery
        <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_add_to_gallery">add_to_gallery</a>&lt;TokenType&gt;(<b>to</b>, to_token);
    }
    // TODO: add event emission
}
</code></pre>



</details>

<a name="0x1_MultiTokenBalance_publish_balance"></a>

## Function `publish_balance`



<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_publish_balance">publish_balance</a>&lt;TokenType: store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_publish_balance">publish_balance</a>&lt;TokenType: store&gt;(account: &signer) {
    <b>assert</b>!(!<b>exists</b>&lt;<a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_EBALANCE_ALREADY_PUBLISHED">EBALANCE_ALREADY_PUBLISHED</a>);
    move_to(account, <a href="MultiTokenBalance.md#0x1_MultiTokenBalance_TokenBalance">TokenBalance</a>&lt;TokenType&gt; { gallery: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>
