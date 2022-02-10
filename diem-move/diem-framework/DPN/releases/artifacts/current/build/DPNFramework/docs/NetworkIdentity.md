
<a name="0x1_NetworkIdentity"></a>

# Module `0x1::NetworkIdentity`

Module managing Diemnet NetworkIdentity


-  [Resource `NetworkIdentity`](#0x1_NetworkIdentity_NetworkIdentity)
-  [Resource `NetworkIdentityEventHandle`](#0x1_NetworkIdentity_NetworkIdentityEventHandle)
-  [Struct `NetworkIdentityChangeNotification`](#0x1_NetworkIdentity_NetworkIdentityChangeNotification)
-  [Constants](#@Constants_0)
-  [Function `initialize_network_identity_event_handle`](#0x1_NetworkIdentity_initialize_network_identity_event_handle)
-  [Function `tc_network_identity_event_handle_exists`](#0x1_NetworkIdentity_tc_network_identity_event_handle_exists)
-  [Function `initialize_network_identity`](#0x1_NetworkIdentity_initialize_network_identity)
-  [Function `get`](#0x1_NetworkIdentity_get)
-  [Function `add_identities`](#0x1_NetworkIdentity_add_identities)
-  [Function `remove_identities`](#0x1_NetworkIdentity_remove_identities)
-  [Function `add_members_internal`](#0x1_NetworkIdentity_add_members_internal)
-  [Function `remove_members_internal`](#0x1_NetworkIdentity_remove_members_internal)


<pre><code><b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NetworkIdentity_NetworkIdentity"></a>

## Resource `NetworkIdentity`

Holder for all <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> in an account


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>identities: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_UniqueMembers">UniqueMembers</a>&lt;vector&lt;u8&gt;&gt; {members: identities};
</code></pre>



</details>

<a name="0x1_NetworkIdentity_NetworkIdentityEventHandle"></a>

## Resource `NetworkIdentityEventHandle`



<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>identity_change_events: <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentity::NetworkIdentityChangeNotification</a>&gt;</code>
</dt>
<dd>
 Event handle for <code>identities</code> rotation events
</dd>
</dl>


</details>

<a name="0x1_NetworkIdentity_NetworkIdentityChangeNotification"></a>

## Struct `NetworkIdentityChangeNotification`

Message sent when there are updates to the <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>.


<pre><code><b>struct</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account: <b>address</b></code>
</dt>
<dd>
 The address of the account that changed identities
</dd>
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



<a name="0x1_NetworkIdentity_ENETWORK_ID_EVENT_HANDLE_INVALID"></a>

Network identity event handle invalid


<pre><code><b>const</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_EVENT_HANDLE_INVALID">ENETWORK_ID_EVENT_HANDLE_INVALID</a>: u64 = 3;
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



<a name="0x1_NetworkIdentity_initialize_network_identity_event_handle"></a>

## Function `initialize_network_identity_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity_event_handle">initialize_network_identity_event_handle</a>(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity_event_handle">initialize_network_identity_event_handle</a>(tc_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account)),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_EVENT_HANDLE_INVALID">ENETWORK_ID_EVENT_HANDLE_INVALID</a>)
    );
    <b>let</b> event_handle = <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a> {
        identity_change_events: <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a>&gt;(tc_account),
    };
    <b>move_to</b>(
        tc_account,
        event_handle,
    );
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_tc_network_identity_event_handle_exists"></a>

## Function `tc_network_identity_event_handle_exists`



<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>(): bool {
    <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(@TreasuryCompliance)
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_initialize_network_identity"></a>

## Function `initialize_network_identity`

Initialize <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> with an empty list


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account: &signer) {
    <b>let</b> identities = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;vector&lt;u8&gt;&gt;();
    <b>move_to</b>(account, <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> { identities });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
</code></pre>



</details>

<a name="0x1_NetworkIdentity_get"></a>

## Function `get`

Return the underlying <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code> bytes


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account_addr: <b>address</b>): vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_get">get</a>(account_addr: <b>address</b>): vector&lt;vector&lt;u8&gt;&gt; <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a> {
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST">ENETWORK_ID_DOESNT_EXIST</a>)
    );
    *&<b>borrow_global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr) <b>with</b> Errors::NOT_PUBLISHED;
<b>ensures</b> result == <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_add_identities"></a>

## Function `add_identities`

Update and create if not exist <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identities">add_identities</a>(account: &signer, to_add: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_identities">add_identities</a>(account: &signer, to_add: vector&lt;vector&lt;u8&gt;&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>, <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a> {
    <b>assert</b>!(<a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>(), <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_EVENT_HANDLE_INVALID">ENETWORK_ID_EVENT_HANDLE_INVALID</a>));
    <b>let</b> num_to_add = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&to_add);
    <b>assert</b>!(num_to_add &gt; 0, <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT">ENETWORK_ID_NO_INPUT</a>));

    <b>if</b> (!<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))) {
        <a href="NetworkIdentity.md#0x1_NetworkIdentity_initialize_network_identity">initialize_network_identity</a>(account);
    };
    <b>let</b> account_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> identity = <b>borrow_global_mut</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
    <b>let</b> identities = &<b>mut</b> identity.identities;

    <b>assert</b>!(
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(identities) + num_to_add &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>,
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED">ENETWORK_ID_LIMIT_EXCEEDED</a>)
    );

    <b>let</b> has_change = <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_members_internal">add_members_internal</a>(identities, &to_add);
    <b>if</b> (has_change) {
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(@TreasuryCompliance).identity_change_events,
            <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
                account: account_addr,
                identities: *&identity.identities,
                time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
            }
        );
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify=<b>false</b>;
<b>let</b> account_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>let</b> prior_identities = <b>if</b> (<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr)) {
    <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities
} <b>else</b> {
    vec()
};
<b>let</b> has_change = (<b>exists</b> e in to_add: !contains(prior_identities, e));
<b>let</b> <b>post</b> handle = <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(@TreasuryCompliance).identity_change_events;
<b>let</b> <b>post</b> msg = <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
    account: account_addr,
    identities: <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities,
    time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_seconds">DiemTimestamp::spec_now_seconds</a>(),
};
<b>aborts_if</b> !<a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>() <b>with</b> Errors::NOT_PUBLISHED;
<b>aborts_if</b> len(to_add) == 0 <b>with</b> Errors::INVALID_ARGUMENT;
<b>aborts_if</b> len(prior_identities) + len(to_add) &gt; MAX_U64;
<b>aborts_if</b> len(prior_identities) + len(to_add) &gt; <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a> <b>with</b> Errors::LIMIT_EXCEEDED;
<b>include</b> has_change ==&gt; <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AddMembersInternalEnsures">AddMembersInternalEnsures</a>&lt;vector&lt;u8&gt;&gt; {
    old_members: prior_identities,
    new_members: <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities,
};
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>emits</b> msg <b>to</b> handle <b>if</b> has_change;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_remove_identities"></a>

## Function `remove_identities`

Remove <code><a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a></code>, skipping if it doesn't exist


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identities">remove_identities</a>(account: &signer, to_remove: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_identities">remove_identities</a>(account: &signer, to_remove: vector&lt;vector&lt;u8&gt;&gt;) <b>acquires</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>, <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a> {
    <b>assert</b>!(<a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>(), <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_EVENT_HANDLE_INVALID">ENETWORK_ID_EVENT_HANDLE_INVALID</a>));
    <b>let</b> num_to_remove = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&to_remove);
    <b>assert</b>!(num_to_remove &gt; 0, <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_NO_INPUT">ENETWORK_ID_NO_INPUT</a>));
    <b>assert</b>!(
        num_to_remove &lt;= <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a>,
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_LIMIT_EXCEEDED">ENETWORK_ID_LIMIT_EXCEEDED</a>)
    );

    <b>let</b> account_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NetworkIdentity.md#0x1_NetworkIdentity_ENETWORK_ID_DOESNT_EXIST">ENETWORK_ID_DOESNT_EXIST</a>)
    );

    <b>let</b> identity = <b>borrow_global_mut</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
    <b>let</b> identities = &<b>mut</b> identity.identities;

    <b>let</b> has_change = <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_members_internal">remove_members_internal</a>(identities, &to_remove);
    <b>if</b> (has_change) {
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(@TreasuryCompliance).identity_change_events,
            <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
                account: account_addr,
                identities: *&identity.identities,
                time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>(),
            }
        );
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> account_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>let</b> prior_identities = <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities;
<b>let</b> has_change = (<b>exists</b> e in to_remove: contains(prior_identities, e));
<b>let</b> <b>post</b> handle = <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityEventHandle">NetworkIdentityEventHandle</a>&gt;(@TreasuryCompliance).identity_change_events;
<b>let</b> <b>post</b> msg = <a href="NetworkIdentity.md#0x1_NetworkIdentity_NetworkIdentityChangeNotification">NetworkIdentityChangeNotification</a> {
    account: account_addr,
    identities: <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities,
    time_rotated_seconds: <a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_seconds">DiemTimestamp::spec_now_seconds</a>(),
};
<b>aborts_if</b> !<a href="NetworkIdentity.md#0x1_NetworkIdentity_tc_network_identity_event_handle_exists">tc_network_identity_event_handle_exists</a>() <b>with</b> Errors::NOT_PUBLISHED;
<b>aborts_if</b> len(to_remove) == 0 <b>with</b> Errors::INVALID_ARGUMENT;
<b>aborts_if</b> len(to_remove) &gt; <a href="NetworkIdentity.md#0x1_NetworkIdentity_MAX_ADDR_IDENTITIES">MAX_ADDR_IDENTITIES</a> <b>with</b> Errors::LIMIT_EXCEEDED;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr) <b>with</b> Errors::NOT_PUBLISHED;
<b>include</b> has_change ==&gt; <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_RemoveMembersInternalEnsures">RemoveMembersInternalEnsures</a>&lt;vector&lt;u8&gt;&gt; {
    old_members: prior_identities,
    new_members: <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr).identities,
};
<b>modifies</b> <b>global</b>&lt;<a href="NetworkIdentity.md#0x1_NetworkIdentity">NetworkIdentity</a>&gt;(account_addr);
<b>emits</b> msg <b>to</b> handle <b>if</b> has_change;
</code></pre>



</details>

<a name="0x1_NetworkIdentity_add_members_internal"></a>

## Function `add_members_internal`

Add all elements that appear in <code>to_add</code> into <code>members</code>.

The <code>members</code> argument is essentially a set simulated by a vector, hence
the uniqueness of its elements are guaranteed, before and after the bulk
insertion. The <code>to_add</code> argument, on the other hand, does not guarantee
to be a set and hence can have duplicated elements.


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_members_internal">add_members_internal</a>&lt;T: <b>copy</b>&gt;(members: &<b>mut</b> vector&lt;T&gt;, to_add: &vector&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_add_members_internal">add_members_internal</a>&lt;T: <b>copy</b>&gt;(
    members: &<b>mut</b> vector&lt;T&gt;,
    to_add: &vector&lt;T&gt;,
): bool {
    <b>let</b> num_to_add = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(to_add);
    <b>let</b> num_existing = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(members);

    <b>let</b> i = 0;
    <b>while</b> ({
        <b>spec</b> {
            <b>invariant</b> i &lt;= num_to_add;
            // the set can never reduce in size
            <b>invariant</b> len(members) &gt;= len(<b>old</b>(members));
            // the current set maintains the uniqueness of the elements
            <b>invariant</b> <b>forall</b> j in 0..len(members), k in 0..len(members): members[j] == members[k] ==&gt; j == k;
            // the left-split of the current set is exactly the same <b>as</b> the original set
            <b>invariant</b> <b>forall</b> j in 0..len(<b>old</b>(members)): members[j] == <b>old</b>(members)[j];
            // all elements in the the right-split of the current set is from the `to_add` vector
            <b>invariant</b> <b>forall</b> j in len(<b>old</b>(members))..len(members): contains(to_add[0..i], members[j]);
            // the current set includes everything in `to_add` we have seen so far
            <b>invariant</b> <b>forall</b> j in 0..i: contains(members, to_add[j]);
            // having no new members means that all elements in the `to_add` vector we have seen so far are already
            // in the existing set, and vice versa.
            <b>invariant</b> len(members) == len(<b>old</b>(members)) &lt;==&gt; (<b>forall</b> j in 0..i: contains(<b>old</b>(members), to_add[j]));
        };
        (i &lt; num_to_add)
    }) {
        <b>let</b> entry = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(to_add, i);
        <b>if</b> (!<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(members, entry)) {
            <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(members, *entry);
        };
        i = i + 1;
    };

    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(members) &gt; num_existing
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>pragma</b> verify=<b>false</b>;
<b>ensures</b> [concrete] <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AddMembersInternalEnsures">AddMembersInternalEnsures</a>&lt;T&gt; {
    old_members: <b>old</b>(members),
    new_members: members,
};
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_UniqueMembers">UniqueMembers</a>&lt;T&gt;;
<b>ensures</b> result == (<b>exists</b> e in to_add: !contains(<b>old</b>(members), e));
</code></pre>




<a name="0x1_NetworkIdentity_AddMembersInternalEnsures"></a>


<pre><code><b>schema</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_AddMembersInternalEnsures">AddMembersInternalEnsures</a>&lt;T&gt; {
    old_members: vector&lt;T&gt;;
    new_members: vector&lt;T&gt;;
    to_add: vector&lt;T&gt;;
    <b>ensures</b> <b>forall</b> e in to_add: contains(new_members, e);
    <b>ensures</b> <b>forall</b> e in old_members: contains(new_members, e);
    <b>ensures</b> <b>forall</b> e in new_members: (contains(old_members, e) || contains(to_add, e));
}
</code></pre>



</details>

<a name="0x1_NetworkIdentity_remove_members_internal"></a>

## Function `remove_members_internal`

Remove all elements that appear in <code>to_remove</code> from <code>members</code>.

The <code>members</code> argument is essentially a set simulated by a vector, hence
the uniqueness of its elements are guaranteed, before and after the bulk
removal. The <code>to_remove</code> argument, on the other hand, does not guarantee
to be a set and hence can have duplicated elements.


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_members_internal">remove_members_internal</a>&lt;T: drop&gt;(members: &<b>mut</b> vector&lt;T&gt;, to_remove: &vector&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_remove_members_internal">remove_members_internal</a>&lt;T: drop&gt;(
    members: &<b>mut</b> vector&lt;T&gt;,
    to_remove: &vector&lt;T&gt;,
): bool {
    <b>let</b> num_existing = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(members);
    <b>let</b> num_to_remove = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(to_remove);

    <b>let</b> i = 0;
    <b>while</b> ({
        <b>spec</b> {
            <b>invariant</b> i &lt;= num_to_remove;
            // the set can never grow in size
            <b>invariant</b> len(members) &lt;= len(<b>old</b>(members));
            // the current set maintains the uniqueness of the elements
            <b>invariant</b> <b>forall</b> j in 0..len(members), k in 0..len(members): members[j] == members[k] ==&gt; j == k;
            // all elements in the the current set come from the original set
            <b>invariant</b> <b>forall</b> j in 0..len(members): contains(<b>old</b>(members), members[j]);
            // the current set never contains anything from the `to_remove` vector
            <b>invariant</b> <b>forall</b> j in 0..i: !contains(members, to_remove[j]);
            // the current set should never remove an element from the original set which is not in `to_remove`
            <b>invariant</b> <b>forall</b> j in 0..len(<b>old</b>(members)): (contains(to_remove[0..i], <b>old</b>(members)[j]) || contains(members, <b>old</b>(members)[j]));
            // having the same member means that all elements in the `to_remove` vector we have seen so far are not
            // in the existing set, and vice versa.
            <b>invariant</b> len(members) == len(<b>old</b>(members)) &lt;==&gt; (<b>forall</b> j in 0..i: !contains(<b>old</b>(members), to_remove[j]));
        };
        (i &lt; num_to_remove)
    }) {
        <b>let</b> entry = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(to_remove, i);
        <b>let</b> (exist, index) = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_index_of">Vector::index_of</a>(members, entry);
        <b>if</b> (exist) {
            <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_swap_remove">Vector::swap_remove</a>(members, index);
        };
        i = i + 1;
    };

    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(members) &lt; num_existing
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> opaque;
<b>ensures</b> [concrete] <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_RemoveMembersInternalEnsures">RemoveMembersInternalEnsures</a>&lt;T&gt; {
    old_members: <b>old</b>(members),
    new_members: members,
};
<b>include</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_UniqueMembers">UniqueMembers</a>&lt;T&gt;;
<b>ensures</b> result == (<b>exists</b> e in to_remove: contains(<b>old</b>(members), e));
</code></pre>




<a name="0x1_NetworkIdentity_RemoveMembersInternalEnsures"></a>


<pre><code><b>schema</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_RemoveMembersInternalEnsures">RemoveMembersInternalEnsures</a>&lt;T&gt; {
    old_members: vector&lt;T&gt;;
    new_members: vector&lt;T&gt;;
    to_remove: vector&lt;T&gt;;
    <b>ensures</b> <b>forall</b> e in to_remove: !contains(new_members, e);
    <b>ensures</b> <b>forall</b> e in new_members: contains(old_members, e);
    <b>ensures</b> <b>forall</b> e in old_members: (contains(to_remove, e) || contains(new_members, e));
}
</code></pre>




<a name="0x1_NetworkIdentity_UniqueMembers"></a>


<pre><code><b>schema</b> <a href="NetworkIdentity.md#0x1_NetworkIdentity_UniqueMembers">UniqueMembers</a>&lt;T&gt; {
    members: vector&lt;T&gt;;
    <b>invariant</b> <b>forall</b> i in 0..len(members), j in 0..len(members): members[i] == members[j] ==&gt; i == j;
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
