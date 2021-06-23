
<a name="0x1_NetworkIdentity"></a>

# Module `0x1::NetworkIdentity`

Module managing Diemnet NetworkIdentity


-  [Resource `NetworkIdentity`](#0x1_NetworkIdentity_NetworkIdentity)
-  [Struct `NetworkIdentityChangeNotification`](#0x1_NetworkIdentity_NetworkIdentityChangeNotification)
-  [Constants](#@Constants_0)
-  [Function `initialize_network_identity`](#0x1_NetworkIdentity_initialize_network_identity)
-  [Function `get`](#0x1_NetworkIdentity_get)
-  [Function `add_identities`](#0x1_NetworkIdentity_add_identities)
-  [Function `add_identity`](#0x1_NetworkIdentity_add_identity)
-  [Function `remove_identities`](#0x1_NetworkIdentity_remove_identities)
-  [Function `remove_identity`](#0x1_NetworkIdentity_remove_identity)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NetworkIdentity_NetworkIdentity"></a>

## Resource `NetworkIdentity`

Holder for all <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> in an account


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>identities: vector&lt;vector&lt;u8&gt;&gt;</code>
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
<code>identities: vector&lt;vector&lt;u8&gt;&gt;</code>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST"></a>

Network ID doesn't exist when trying to get it


<pre><code><b>const</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST">ENETWORK_ID_DOESNT_EXIST</a>: u64 = 0;
</code></pre>



<a name="0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED"></a>

Limit exceeded on number of identities for an address


<pre><code><b>const</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED">ENETWORK_ID_LIMIT_EXCEEDED</a>: u64 = 1;
</code></pre>



<a name="0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT"></a>

No identities provided for changes


<pre><code><b>const</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT">ENETWORK_ID_NO_INPUT</a>: u64 = 2;
</code></pre>



<a name="0x1_NetworkIdentity_MAX_ADDR_IDENTITIES"></a>



<pre><code><b>const</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>: u64 = 100;
</code></pre>



<a name="0x1_NetworkIdentity_initialize_network_identity"></a>

## Function `initialize_network_identity`

Initialize <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> with an empty list


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account: &signer) {
    <b>let</b> identities = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;vector&lt;u8&gt;&gt;();
    <b>let</b> identity_change_events = <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a>&gt;(account);
    move_to(account, <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> { identities, identity_change_events });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>ensures</b> <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_get"></a>

## Function `get`

Return the underlying <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> bytes


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account_addr: address): vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account_addr: address): vector&lt;vector&lt;u8&gt;&gt; <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    <b>assert</b>(<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr), <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST">ENETWORK_ID_DOESNT_EXIST</a>);
    *&borrow_global&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_add_identities"></a>

## Function `add_identities`

Update and create if not exist <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identities">add_identities</a>(account: &signer, to_add: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identities">add_identities</a>(account: &signer, to_add: vector&lt;vector&lt;u8&gt;&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    <b>let</b> num_to_add = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&to_add);
    <b>assert</b>(num_to_add &gt; 0, <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT">ENETWORK_ID_NO_INPUT</a>);

    <b>if</b> (!<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account);
    };
    <b>let</b> identity = borrow_global_mut&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>let</b> identities = &<b>mut</b> identity.identities;

    <b>assert</b>(<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(identities) + num_to_add &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>, <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED">ENETWORK_ID_LIMIT_EXCEEDED</a>));

    <b>let</b> i = 0;
    <b>let</b> has_change = <b>false</b>;
    <b>while</b> (i &lt; num_to_add) {
       has_change = has_change || <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identity">add_identity</a>(identities, *<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&to_add, i));
       i = i + 1;
    };


    <b>if</b> (has_change) {
        <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> identity.identity_change_events, <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
            identities: *&identity.identities,
            time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
        });
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>let</b> num_identities = len(<b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities);
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>invariant</b> <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>invariant</b> num_identities &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_add_identity"></a>

## Function `add_identity`

Adds an identity and returns true if a change was made


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identity">add_identity</a>(identities: &<b>mut</b> vector&lt;vector&lt;u8&gt;&gt;, to_add: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identity">add_identity</a>(identities: &<b>mut</b> vector&lt;vector&lt;u8&gt;&gt;, to_add: vector&lt;u8&gt;): bool {
    <b>if</b> (!<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(identities, &to_add)) {
        <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(identities, to_add);
        <b>true</b>
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>ensures</b> contains&lt;vector&lt;u8&gt;&gt;(identities, to_add);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_remove_identities"></a>

## Function `remove_identities`

Remove <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>, skipping if it doesn't exist


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identities">remove_identities</a>(account: &signer, to_remove: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identities">remove_identities</a>(account: &signer, to_remove: vector&lt;vector&lt;u8&gt;&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    <b>let</b> num_to_remove = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&to_remove);
    <b>assert</b>(num_to_remove &gt; 0, <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT">ENETWORK_ID_NO_INPUT</a>);
    <b>assert</b>(num_to_remove &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>, <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED">ENETWORK_ID_LIMIT_EXCEEDED</a>);

    <b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr), <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST">ENETWORK_ID_DOESNT_EXIST</a>);

    <b>let</b> identity = borrow_global_mut&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
    <b>let</b> identities = &<b>mut</b> identity.identities;

    <b>let</b> i = 0;
    <b>let</b> has_change = <b>false</b>;
    <b>while</b> (i &lt; num_to_remove) {
       has_change = has_change || <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identity">remove_identity</a>(identities, *<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&to_remove, i));
       i = i + 1;
    };

    <b>if</b> (has_change) {
        <a href="../../../../../../move-stdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> identity.identity_change_events, <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
            identities: *&identity.identities,
            time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
        });
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>let</b> num_identities = len(<b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities);
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>invariant</b> <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>invariant</b> num_identities &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_remove_identity"></a>

## Function `remove_identity`

Removes an identity and returns true if a change was made


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identity">remove_identity</a>(identities: &<b>mut</b> vector&lt;vector&lt;u8&gt;&gt;, to_remove: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identity">remove_identity</a>(identities: &<b>mut</b> vector&lt;vector&lt;u8&gt;&gt;, to_remove: vector&lt;u8&gt;): bool {
    <b>let</b> (exist, i) = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_index_of">Vector::index_of</a>(identities, &to_remove);

    <b>if</b> (exist) {
        <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_swap_remove">Vector::swap_remove</a>(identities, i);
    };

    exist
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
