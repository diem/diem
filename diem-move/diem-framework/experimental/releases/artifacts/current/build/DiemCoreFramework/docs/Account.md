
<a name="0x1_Account"></a>

# Module `0x1::Account`



-  [Resource `Account`](#0x1_Account_Account)
-  [Resource `Marker`](#0x1_Account_Marker)
-  [Resource `ChainSpecificAccountInfo`](#0x1_Account_ChainSpecificAccountInfo)
-  [Constants](#@Constants_0)
-  [Function `create_signer`](#0x1_Account_create_signer)
-  [Function `initialize`](#0x1_Account_initialize)
-  [Function `assert_is_marker`](#0x1_Account_assert_is_marker)
-  [Function `create_authentication_key`](#0x1_Account_create_authentication_key)
-  [Function `create_account`](#0x1_Account_create_account)
-  [Function `exists_at`](#0x1_Account_exists_at)
-  [Function `get_sequence_number`](#0x1_Account_get_sequence_number)
-  [Function `get_authentication_key`](#0x1_Account_get_authentication_key)
-  [Function `rotate_authentication_key`](#0x1_Account_rotate_authentication_key)
-  [Function `prologue`](#0x1_Account_prologue)
-  [Function `epilogue`](#0x1_Account_epilogue)
-  [Function `writeset_epilogue`](#0x1_Account_writeset_epilogue)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BCS.md#0x1_BCS">0x1::BCS</a>;
<b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Hash.md#0x1_Hash">0x1::Hash</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Account_Account"></a>

## Resource `Account`

Resource representing an account.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account">Account</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>self_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_Marker"></a>

## Resource `Marker`

A marker resource that registers the type <code>T</code> as the system marker for BasicAccount at genesis.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_Marker">Marker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_Account_ChainSpecificAccountInfo"></a>

## Resource `ChainSpecificAccountInfo`

This holds information that will be picked up by the VM to call the
correct chain-specific prologue and epilogue functions


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_ChainSpecificAccountInfo">ChainSpecificAccountInfo</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>script_prologue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>module_prologue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>writeset_prologue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>multi_agent_prologue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>user_epilogue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>writeset_epilogue_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>currency_code_required: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Account_MAX_U64"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_Account_EACCOUNT"></a>

Account already existed


<pre><code><b>const</b> <a href="Account.md#0x1_Account_EACCOUNT">EACCOUNT</a>: u64 = 0;
</code></pre>



<a name="0x1_Account_EMALFORMED_AUTHENTICATION_KEY"></a>

The provided authentication had an invalid length


<pre><code><b>const</b> <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>: u64 = 4;
</code></pre>



<a name="0x1_Account_ENOT_CORE_FRAMEWORK"></a>

The address provided didn't match the <code>CoreFramework</code> address.


<pre><code><b>const</b> <a href="Account.md#0x1_Account_ENOT_CORE_FRAMEWORK">ENOT_CORE_FRAMEWORK</a>: u64 = 2;
</code></pre>



<a name="0x1_Account_ENOT_MARKER_TYPE"></a>

The marker type provided is not the registered type for <code><a href="Account.md#0x1_Account">Account</a></code>.


<pre><code><b>const</b> <a href="Account.md#0x1_Account_ENOT_MARKER_TYPE">ENOT_MARKER_TYPE</a>: u64 = 3;
</code></pre>



<a name="0x1_Account_ESEQUENCE_NUMBER_TOO_BIG"></a>

Sequence number exceeded the maximum value for a u64


<pre><code><b>const</b> <a href="Account.md#0x1_Account_ESEQUENCE_NUMBER_TOO_BIG">ESEQUENCE_NUMBER_TOO_BIG</a>: u64 = 1;
</code></pre>



<a name="0x1_Account_PROLOGUE_EACCOUNT_DNE"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_EACCOUNT_DNE">PROLOGUE_EACCOUNT_DNE</a>: u64 = 1004;
</code></pre>



<a name="0x1_Account_PROLOGUE_EBAD_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_EBAD_CHAIN_ID">PROLOGUE_EBAD_CHAIN_ID</a>: u64 = 1005;
</code></pre>



<a name="0x1_Account_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>: u64 = 1001;
</code></pre>



<a name="0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG">PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG</a>: u64 = 1006;
</code></pre>



<a name="0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>: u64 = 1003;
</code></pre>



<a name="0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>: u64 = 1002;
</code></pre>



<a name="0x1_Account_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: <b>address</b>): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: <b>address</b>): signer;
</code></pre>



</details>

<a name="0x1_Account_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_initialize">initialize</a>&lt;T&gt;(account: &signer, module_addr: <b>address</b>, module_name: vector&lt;u8&gt;, script_prologue_name: vector&lt;u8&gt;, module_prologue_name: vector&lt;u8&gt;, writeset_prologue_name: vector&lt;u8&gt;, multi_agent_prologue_name: vector&lt;u8&gt;, user_epilogue_name: vector&lt;u8&gt;, writeset_epilogue_name: vector&lt;u8&gt;, currency_code_required: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_initialize">initialize</a>&lt;T&gt;(account: &signer,
    module_addr: <b>address</b>,
    module_name: vector&lt;u8&gt;,
    script_prologue_name: vector&lt;u8&gt;,
    module_prologue_name: vector&lt;u8&gt;,
    writeset_prologue_name: vector&lt;u8&gt;,
    multi_agent_prologue_name: vector&lt;u8&gt;,
    user_epilogue_name: vector&lt;u8&gt;,
    writeset_epilogue_name: vector&lt;u8&gt;,
    currency_code_required: bool,
) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == @CoreResources, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Account.md#0x1_Account_ENOT_CORE_FRAMEWORK">ENOT_CORE_FRAMEWORK</a>));
    <b>move_to</b>(account, <a href="Account.md#0x1_Account_Marker">Marker</a>&lt;T&gt; {});
    <b>move_to</b>(account, <a href="Account.md#0x1_Account_ChainSpecificAccountInfo">ChainSpecificAccountInfo</a> {
        module_addr,
        module_name,
        script_prologue_name,
        module_prologue_name,
        writeset_prologue_name,
        multi_agent_prologue_name,
        user_epilogue_name,
        writeset_epilogue_name,
        currency_code_required,
    });
}
</code></pre>



</details>

<a name="0x1_Account_assert_is_marker"></a>

## Function `assert_is_marker`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_assert_is_marker">assert_is_marker</a>&lt;T&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_assert_is_marker">assert_is_marker</a>&lt;T&gt;() {
    <b>assert</b>!(<b>exists</b>&lt;<a href="Account.md#0x1_Account_Marker">Marker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_ENOT_MARKER_TYPE">ENOT_MARKER_TYPE</a>))
}
</code></pre>



</details>

<a name="0x1_Account_create_authentication_key"></a>

## Function `create_authentication_key`

Construct an authentication key, aborting if the prefix is not valid.


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_create_authentication_key">create_authentication_key</a>(account: &signer, auth_key_prefix: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_create_authentication_key">create_authentication_key</a>(account: &signer, auth_key_prefix: vector&lt;u8&gt;): vector&lt;u8&gt; {
    <b>let</b> authentication_key = auth_key_prefix;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_append">Vector::append</a>(
        &<b>mut</b> authentication_key, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_borrow_address">Signer::borrow_address</a>(account))
    );
    <b>assert</b>!(
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );
    authentication_key
}
</code></pre>



</details>

<a name="0x1_Account_create_account"></a>

## Function `create_account`

Publishes a new <code><a href="Account.md#0x1_Account">Account</a></code> resource under <code>new_address</code>.
A signer representing <code>new_address</code> is returned. This way, the caller of this function
can publish additional resources under <code>new_address</code>.
The <code>_witness</code> guarantees that owner the registered caller of this function can call it.
authentication key returned is <code>auth_key_prefix</code> | <code>fresh_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;T&gt;(new_address: <b>address</b>, authentication_key_prefix: vector&lt;u8&gt;, _witness: &T): (signer, vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;T&gt;(
    new_address: <b>address</b>,
    authentication_key_prefix: vector&lt;u8&gt;,
    _witness: &T,
): (signer, vector&lt;u8&gt;) {
    <a href="Account.md#0x1_Account_assert_is_marker">assert_is_marker</a>&lt;T&gt;();
    // there cannot be an <a href="Account.md#0x1_Account">Account</a> resource under new_addr already.
    <b>assert</b>!(!<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(new_address), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="Account.md#0x1_Account_EACCOUNT">EACCOUNT</a>));

    <b>let</b> new_account = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(new_address);
    <b>let</b> authentication_key = <a href="Account.md#0x1_Account_create_authentication_key">create_authentication_key</a>(&new_account, authentication_key_prefix);
    <b>move_to</b>(
        &new_account,
        <a href="Account.md#0x1_Account">Account</a> {
            authentication_key: <b>copy</b> authentication_key,
            sequence_number: 0,
            self_address: new_address,
        }
    );

    (new_account, authentication_key)
}
</code></pre>



</details>

<a name="0x1_Account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Account_get_sequence_number"></a>

## Function `get_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_get_sequence_number">get_sequence_number</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_get_sequence_number">get_sequence_number</a>(addr: <b>address</b>) : u64 <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).sequence_number
}
</code></pre>



</details>

<a name="0x1_Account_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_get_authentication_key">get_authentication_key</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_get_authentication_key">get_authentication_key</a>(addr: <b>address</b>) : vector&lt;u8&gt; <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    *&<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(
    account: &signer,
    new_auth_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<a href="Account.md#0x1_Account_exists_at">exists_at</a>(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Account.md#0x1_Account_EACCOUNT">EACCOUNT</a>));
    <b>assert</b>!(
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&new_auth_key) == 32,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
    account_resource.authentication_key = new_auth_key;
}
</code></pre>



</details>

<a name="0x1_Account_prologue"></a>

## Function `prologue`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_prologue">prologue</a>(account: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_prologue">prologue</a>(
    account: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    chain_id: u8,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> transaction_sender = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_PROLOGUE_EBAD_CHAIN_ID">PROLOGUE_EBAD_CHAIN_ID</a>));
    <b>assert</b>!(<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(transaction_sender), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_PROLOGUE_EACCOUNT_DNE">PROLOGUE_EACCOUNT_DNE</a>));
    <b>let</b> sender_account = <b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(transaction_sender);
    <b>assert</b>!(
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>),
    );
    <b>assert</b>!(
        (txn_sequence_number <b>as</b> u128) &lt; <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG">PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG</a>)
    );

    <b>assert</b>!(
        txn_sequence_number &gt;= sender_account.sequence_number,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>)
    );

    // [PCA12]: Check that the transaction's sequence number matches the
    // current sequence number. Otherwise sequence number is too new by [PCA11].
    <b>assert</b>!(
        txn_sequence_number == sender_account.sequence_number,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>)
    );
}
</code></pre>



</details>

<a name="0x1_Account_epilogue"></a>

## Function `epilogue`

Epilogue function is run after a transaction is successfully executed.
Called by the Adaptor


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_epilogue">epilogue</a>&lt;T&gt;(account: &signer, _witness: &T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_epilogue">epilogue</a>&lt;T&gt;(account: &signer, _witness: &T) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Account.md#0x1_Account_assert_is_marker">assert_is_marker</a>&lt;T&gt;();
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> old_sequence_number = <a href="Account.md#0x1_Account_get_sequence_number">get_sequence_number</a>(addr);

    <b>assert</b>!(
        (old_sequence_number <b>as</b> u128) &lt; <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="Account.md#0x1_Account_ESEQUENCE_NUMBER_TOO_BIG">ESEQUENCE_NUMBER_TOO_BIG</a>)
    );

    // Increment sequence number
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
    account_resource.sequence_number = old_sequence_number + 1;
}
</code></pre>



</details>

<a name="0x1_Account_writeset_epilogue"></a>

## Function `writeset_epilogue`

Epilogue function called after a successful writeset transaction, which can only be sent by @CoreResources.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_writeset_epilogue">writeset_epilogue</a>&lt;T&gt;(account: &signer, should_trigger_reconfiguration: bool, witness: &T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_writeset_epilogue">writeset_epilogue</a>&lt;T&gt;(
    account: &signer,
    should_trigger_reconfiguration: bool,
    witness: &T
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);
    <a href="Account.md#0x1_Account_epilogue">epilogue</a>(account, witness);
    <b>if</b> (should_trigger_reconfiguration) <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>
