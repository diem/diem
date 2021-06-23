
<a name="0x1_NetworkIdentity"></a>

# Module `0x1::NetworkIdentity`

Module managing Diemnet NetworkIdentity


-  [Resource `AccountList`](#0x1_NetworkIdentity_AccountList)
-  [Struct `AccountListChangeNotification`](#0x1_NetworkIdentity_AccountListChangeNotification)
-  [Resource `NetworkIdentity`](#0x1_NetworkIdentity_NetworkIdentity)
-  [Struct `NetworkIdentityChangeNotification`](#0x1_NetworkIdentity_NetworkIdentityChangeNotification)
-  [Function `initialize`](#0x1_NetworkIdentity_initialize)
-  [Function `get`](#0x1_NetworkIdentity_get)
-  [Function `update_identities`](#0x1_NetworkIdentity_update_identities)
-  [Function `update_accounts`](#0x1_NetworkIdentity_update_accounts)
-  [Module Specification](#@Module_Specification_0)


<pre><code><b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NetworkIdentity_AccountList"></a>

## Resource `AccountList`

An updatable <code>address</code> list with update notifications


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>accounts: vector&lt;address&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>account_change_events: <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountListChangeNotification">NetworkIdentity::AccountListChangeNotification</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_AccountListChangeNotification"></a>

## Struct `AccountListChangeNotification`

Message sent when there are updates to the <code><a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a></code>.


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountListChangeNotification">AccountListChangeNotification</a> has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>new_accounts: vector&lt;address&gt;</code>
</dt>
<dd>
 The new accounts
</dd>
<dt>
<code>time_rotated_seconds: u64</code>
</dt>
<dd>
 The time at which the <code>accounts</code> was rotated
</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_NetworkIdentity"></a>

## Resource `NetworkIdentity`

Holder for all <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> in an account


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>identities: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>identity_change_events: <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentity::NetworkIdentityChangeNotification</a>&gt;</code>
</dt>
<dd>
 Event handle for <code>identities</code> rotation events
</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_NetworkIdentityChangeNotification"></a>

## Struct `NetworkIdentityChangeNotification`

Message sent when there are updates to the <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>.


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>new_identities: vector&lt;u8&gt;</code>
</dt>
<dd>
 The new identities
</dd>
<dt>
<code>time_rotated_seconds: u64</code>
</dt>
<dd>
 The time at which the <code>identities</code> was rotated
</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_initialize"></a>

## Function `initialize`

Initialize <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> with an empty list


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account: &signer) {
    <b>let</b> identities = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;u8&gt;();
    <b>let</b> identity_change_events = <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a>&gt;(account);
    move_to(account, <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> { identities, identity_change_events });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>let</b> addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(addr);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_get"></a>

## Function `get`

Return the underlying <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> bytes


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account: address): vector&lt;u8&gt; <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    *&borrow_global&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account).identities
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_update_identities"></a>

## Function `update_identities`

Update and create if not exist <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_update_identities">update_identities</a>(account: &signer, identities: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_update_identities">update_identities</a>(account: &signer, identities: vector&lt;u8&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize">initialize</a>(account);
    };
    <b>let</b> holder = borrow_global_mut&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    holder.identities = <b>copy</b> identities;

    <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> holder.identity_change_events, <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
        new_identities: identities,
        time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
    });
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_update_accounts"></a>

## Function `update_accounts`

Update and create if not exist <code><a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_update_accounts">update_accounts</a>(account: &signer, accounts: vector&lt;address&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_update_accounts">update_accounts</a>(account: &signer, accounts: vector&lt;address&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
			<b>let</b> accounts = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;address&gt;();
			<b>let</b> account_change_events = <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountListChangeNotification">AccountListChangeNotification</a>&gt;(account);
			move_to(account, <a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a> { accounts, account_change_events });
    };
    <b>let</b> holder = borrow_global_mut&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountList">AccountList</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    holder.accounts = <b>copy</b> accounts;

    <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> holder.account_change_events, <a href="NetworkIdentity.md#0x1_NetworkIdentity_AccountListChangeNotification">AccountListChangeNotification</a> {
        new_accounts: accounts,
        time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
    });
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
