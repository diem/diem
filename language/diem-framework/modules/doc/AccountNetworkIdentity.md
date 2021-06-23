
<a name="0x1_NetworkIdentity"></a>

# Module `0x1::NetworkIdentity`



-  [Struct `AddressList`](#0x1_NetworkIdentity_AddressList)
-  [Struct `PublicKey`](#0x1_NetworkIdentity_PublicKey)
-  [Struct `NetworkIdentity`](#0x1_NetworkIdentity_NetworkIdentity)
-  [Resource `NetworkIdentities`](#0x1_NetworkIdentity_NetworkIdentities)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_NetworkIdentity_initialize)
-  [Function `get`](#0x1_NetworkIdentity_get)
-  [Function `ensure_exists`](#0x1_NetworkIdentity_ensure_exists)
-  [Function `add_identity`](#0x1_NetworkIdentity_add_identity)
-  [Function `update_identities`](#0x1_NetworkIdentity_update_identities)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)


<pre><code><b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NetworkIdentity_AddressList"></a>

## Struct `AddressList`

A list of addresses to watch


<pre><code><b>struct</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_AddressList">AddressList</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addresses: vector&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_PublicKey"></a>

## Struct `PublicKey`

A byte representation of a x25519 Public Key


<pre><code><b>struct</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_PublicKey">PublicKey</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>bytes: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<details>
<summary>Specification</summary>


All <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_PublicKey">PublicKey</a></code>s must be exactly 32 characters long.


<pre><code><b>invariant</b> len(bytes) == <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_KEY_LENGTH">KEY_LENGTH</a>;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_NetworkIdentity"></a>

## Struct `NetworkIdentity`

A combination of peer id and public keys that are valid together


<pre><code><b>struct</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>peer_id: address</code>
</dt>
<dd>

</dd>
<dt>
<code>public_key: vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_PublicKey">NetworkIdentity::PublicKey</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_NetworkIdentities"></a>

## Resource `NetworkIdentities`

Holder for all <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> in an account


<pre><code><b>struct</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a> has <b>copy</b>, drop, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>identities: vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentity">NetworkIdentity::NetworkIdentity</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_NetworkIdentity_EALREADY_PUBLISHED"></a>

Already initialized NetworkIdentities


<pre><code><b>const</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_EALREADY_PUBLISHED">EALREADY_PUBLISHED</a>: u64 = 2;
</code></pre>



<a name="0x1_NetworkIdentity_EPEER_ID"></a>

The <code>PeerId</code> was invalid


<pre><code><b>const</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_EPEER_ID">EPEER_ID</a>: u64 = 1;
</code></pre>



<a name="0x1_NetworkIdentity_EPUBLIC_KEY"></a>

The <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_PublicKey">PublicKey</a></code> was invalid


<pre><code><b>const</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_EPUBLIC_KEY">EPUBLIC_KEY</a>: u64 = 0;
</code></pre>



<a name="0x1_NetworkIdentity_KEY_LENGTH"></a>



<pre><code><b>const</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_KEY_LENGTH">KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_NetworkIdentity_initialize"></a>

## Function `initialize`

Initialize <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a></code> with an empty list


<pre><code><b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account: &signer) {
    <b>assert</b>(!<b>exists</b>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_EALREADY_PUBLISHED">EALREADY_PUBLISHED</a>));
    <b>let</b> identities = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;();
    move_to(account, <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a> { identities })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>let</b> addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>modifies</b> <b>global</b>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(addr) <b>with</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <b>exists</b>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(addr);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_get"></a>

## Function `get`

Return the <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account: address): vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentity">NetworkIdentity::NetworkIdentity</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account: address): vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt; <b>acquires</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a> {
    *&borrow_global&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(account).identities
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_ensure_exists"></a>

## Function `ensure_exists`

Ensure <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a></code> exists and create if it doesn't


<pre><code><b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_ensure_exists">ensure_exists</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_ensure_exists">ensure_exists</a>(account: &signer) {
    <b>if</b> (!<b>exists</b>&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account);
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



</details>

<a name="0x1_NetworkIdentity_add_identity"></a>

## Function `add_identity`

Add a single <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>, doesn't know anything about previous identities


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_add_identity">add_identity</a>(account: &signer, network_identity: <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentity">NetworkIdentity::NetworkIdentity</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_add_identity">add_identity</a>(account: &signer, network_identity: <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>) <b>acquires</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a> {
    <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_ensure_exists">ensure_exists</a>(account);
    <b>let</b> identities = &<b>mut</b> borrow_global_mut&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).identities;
    <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(identities, network_identity);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



</details>

<a name="0x1_NetworkIdentity_update_identities"></a>

## Function `update_identities`

In place replacement of entirety of <code><a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>s


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_update_identities">update_identities</a>(account: &signer, identities: vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentity">NetworkIdentity::NetworkIdentity</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_update_identities">update_identities</a>(account: &signer, identities: vector&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;) <b>acquires</b> <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a> {
    <a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_ensure_exists">ensure_exists</a>(account);
    <b>let</b> holder = borrow_global_mut&lt;<a href="AccountNetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentities">NetworkIdentities</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
		holder.identities = identities;
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
