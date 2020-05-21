
<a name="0x0_AccountType"></a>

# Module `0x0::AccountType`

### Table of Contents

-  [Struct `T`](#0x0_AccountType_T)
-  [Struct `TransitionCapability`](#0x0_AccountType_TransitionCapability)
-  [Struct `GrantingCapability`](#0x0_AccountType_GrantingCapability)
-  [Struct `UpdateCapability`](#0x0_AccountType_UpdateCapability)
-  [Struct `Registered`](#0x0_AccountType_Registered)
-  [Function `grant_account_tracking`](#0x0_AccountType_grant_account_tracking)
-  [Function `create`](#0x0_AccountType_create)
-  [Function `apply_for`](#0x0_AccountType_apply_for)
-  [Function `transition`](#0x0_AccountType_transition)
-  [Function `is_a`](#0x0_AccountType_is_a)
-  [Function `root_address`](#0x0_AccountType_root_address)
-  [Function `has_transition_cap`](#0x0_AccountType_has_transition_cap)
-  [Function `transition_cap_root_addr`](#0x0_AccountType_transition_cap_root_addr)
-  [Function `has_granting_cap`](#0x0_AccountType_has_granting_cap)
-  [Function `account_metadata`](#0x0_AccountType_account_metadata)
-  [Function `update`](#0x0_AccountType_update)
-  [Function `update_with_capability`](#0x0_AccountType_update_with_capability)
-  [Function `apply_for_transition_capability`](#0x0_AccountType_apply_for_transition_capability)
-  [Function `apply_for_granting_capability`](#0x0_AccountType_apply_for_granting_capability)
-  [Function `grant_transition_capability`](#0x0_AccountType_grant_transition_capability)
-  [Function `certify_granting_capability`](#0x0_AccountType_certify_granting_capability)
-  [Function `remove_transition_capability`](#0x0_AccountType_remove_transition_capability)
-  [Function `remove_granting_capability_from_sender`](#0x0_AccountType_remove_granting_capability_from_sender)
-  [Function `remove_granting_capability`](#0x0_AccountType_remove_granting_capability)
-  [Function `register`](#0x0_AccountType_register)
-  [Function `assert_is_a`](#0x0_AccountType_assert_is_a)
-  [Function `assert_has_transition_cap`](#0x0_AccountType_assert_has_transition_cap)
-  [Function `singleton_addr`](#0x0_AccountType_singleton_addr)
-  [Function `assert_is_registered`](#0x0_AccountType_assert_is_registered)



<a name="0x0_AccountType_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>is_certified: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>account_metadata: <a href="#0x0_AccountType">AccountType</a></code>
</dt>
<dd>

</dd>
<dt>

<code>root_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountType_TransitionCapability"></a>

## Struct `TransitionCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>is_certified: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>root_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountType_GrantingCapability"></a>

## Struct `GrantingCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>is_certified: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountType_UpdateCapability"></a>

## Struct `UpdateCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountType_UpdateCapability">UpdateCapability</a>
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

<a name="0x0_AccountType_Registered"></a>

## Struct `Registered`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountType_Registered">Registered</a>&lt;Type: <b>copyable</b>&gt;
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

<a name="0x0_AccountType_grant_account_tracking"></a>

## Function `grant_account_tracking`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_grant_account_tracking">grant_account_tracking</a>(): <a href="#0x0_AccountType_UpdateCapability">AccountType::UpdateCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_grant_account_tracking">grant_account_tracking</a>(): <a href="#0x0_AccountType_UpdateCapability">UpdateCapability</a> {
    // This needs <b>to</b> match the singleton_addr in <a href="account_tracking.md#0x0_AccountTrack">AccountTrack</a>
    Transaction::assert(Transaction::sender() == 0xA550C18, 2006);
    <a href="#0x0_AccountType_UpdateCapability">UpdateCapability</a>{}
}
</code></pre>



</details>

<a name="0x0_AccountType_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_create">create</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(fresh_address: address, account_metadata: <a href="#0x0_AccountType">AccountType</a>): <a href="#0x0_AccountType_T">AccountType::T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_create">create</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(
    fresh_address: address,
    account_metadata: <a href="#0x0_AccountType">AccountType</a>
): <a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt; {
    <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;();
    <a href="#0x0_AccountType_T">T</a> {
        account_metadata,
        is_certified: <b>true</b>,
        root_address: fresh_address,
    }
}
</code></pre>



</details>

<a name="0x0_AccountType_apply_for"></a>

## Function `apply_for`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for">apply_for</a>&lt;Type: <b>copyable</b>&gt;(account_metadata: Type, root_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for">apply_for</a>&lt;Type: <b>copyable</b>&gt;(account_metadata: Type, root_address: address) {
    <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;Type&gt;();
    move_to_sender(<a href="#0x0_AccountType_T">T</a>&lt;Type&gt; {
        is_certified: <b>false</b>,
        account_metadata,
        root_address,
    });
}
</code></pre>



</details>

<a name="0x0_AccountType_transition"></a>

## Function `transition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_transition">transition</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_transition">transition</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>, <a href="#0x0_AccountType_T">T</a> {
    // Make sure the account is an empty account
    <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;<a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;(addr);
    // Get the transition capability held under the sender
    <b>let</b> transition_cap = borrow_global&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(Transaction::sender());
    // Make sure it's certified
    Transaction::assert(transition_cap.is_certified, 2000);
    <b>let</b> <a href="#0x0_AccountType_T">T</a>{ is_certified: _, account_metadata: _, root_address: _ } = move_from&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;&gt;(addr);
    <b>let</b> <b>to</b> = borrow_global_mut&lt;<a href="#0x0_AccountType_T">T</a>&lt;To&gt;&gt;(addr);
    // Make sure that the root address transition capability matches
    // the root address for the published `To` account type.
    Transaction::assert(<b>to</b>.root_address == transition_cap.root_address, 2001);
    <b>to</b>.is_certified = <b>true</b>;
}
</code></pre>



</details>

<a name="0x0_AccountType_is_a"></a>

## Function `is_a`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_is_a">is_a</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_is_a">is_a</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): bool
<b>acquires</b> <a href="#0x0_AccountType_T">T</a> {
    exists&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;&gt;(addr) && borrow_global&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;&gt;(addr).is_certified
}
</code></pre>



</details>

<a name="0x0_AccountType_root_address"></a>

## Function `root_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_root_address">root_address</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_root_address">root_address</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): address
<b>acquires</b> <a href="#0x0_AccountType_T">T</a> {
    <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;(addr);
    borrow_global&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;&gt;(addr).root_address
}
</code></pre>



</details>

<a name="0x0_AccountType_has_transition_cap"></a>

## Function `has_transition_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_has_transition_cap">has_transition_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_has_transition_cap">has_transition_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address): bool
<b>acquires</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> {
    exists&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(addr) &&
        borrow_global&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(addr).is_certified
}
</code></pre>



</details>

<a name="0x0_AccountType_transition_cap_root_addr"></a>

## Function `transition_cap_root_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_transition_cap_root_addr">transition_cap_root_addr</a>&lt;To: <b>copyable</b>&gt;(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_transition_cap_root_addr">transition_cap_root_addr</a>&lt;To: <b>copyable</b>&gt;(addr: address): address
<b>acquires</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> {
    <a href="#0x0_AccountType_assert_has_transition_cap">assert_has_transition_cap</a>&lt;To&gt;(addr);
    borrow_global&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(addr).root_address
}
</code></pre>



</details>

<a name="0x0_AccountType_has_granting_cap"></a>

## Function `has_granting_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_has_granting_cap">has_granting_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_has_granting_cap">has_granting_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address): bool
<b>acquires</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> {
    exists&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(addr) &&
        borrow_global&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(addr).is_certified
}
</code></pre>



</details>

<a name="0x0_AccountType_account_metadata"></a>

## Function `account_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_account_metadata">account_metadata</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): <a href="#0x0_AccountType">AccountType</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_account_metadata">account_metadata</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address): <a href="#0x0_AccountType">AccountType</a>
<b>acquires</b> <a href="#0x0_AccountType_T">T</a> {
    <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;(addr);
    *&borrow_global&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;&gt;(addr).account_metadata
}
</code></pre>



</details>

<a name="0x0_AccountType_update"></a>

## Function `update`



<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;Type: <b>copyable</b>&gt;(addr: address, new_account_metadata: Type)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;Type: <b>copyable</b>&gt;(addr: address, new_account_metadata: Type)
<b>acquires</b> <a href="#0x0_AccountType_T">T</a>, <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> {
    <b>let</b> sender = Transaction::sender();
    <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;Type&gt;(addr);
    // Make sure that the sender has a transition capability.
    <a href="#0x0_AccountType_assert_has_transition_cap">assert_has_transition_cap</a>&lt;Type&gt;(sender);
    <b>let</b> account_type = borrow_global_mut&lt;<a href="#0x0_AccountType_T">T</a>&lt;Type&gt;&gt;(addr);
    <b>let</b> transition_cap = borrow_global&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;Type&gt;&gt;(sender);
    // Make sure that the transition capability and the account at
    // `addr` have the same root of authority.
    Transaction::assert(account_type.root_address == transition_cap.root_address, 2001);
    account_type.account_metadata = new_account_metadata;
}
</code></pre>



</details>

<a name="0x0_AccountType_update_with_capability"></a>

## Function `update_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_update_with_capability">update_with_capability</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(addr: address, new_account_metadata: <a href="#0x0_AccountType">AccountType</a>, _update_cap: &<a href="#0x0_AccountType_UpdateCapability">AccountType::UpdateCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_update_with_capability">update_with_capability</a>&lt;<a href="#0x0_AccountType">AccountType</a>: <b>copyable</b>&gt;(
    addr: address,
    new_account_metadata: <a href="#0x0_AccountType">AccountType</a>,
    _update_cap: &<a href="#0x0_AccountType_UpdateCapability">UpdateCapability</a>
) <b>acquires</b> <a href="#0x0_AccountType_T">T</a> {
    <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;(addr);
    <b>let</b> account_type = borrow_global_mut&lt;<a href="#0x0_AccountType_T">T</a>&lt;<a href="#0x0_AccountType">AccountType</a>&gt;&gt;(addr);
    account_type.account_metadata = new_account_metadata;
}
</code></pre>



</details>

<a name="0x0_AccountType_apply_for_transition_capability"></a>

## Function `apply_for_transition_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for_transition_capability">apply_for_transition_capability</a>&lt;To: <b>copyable</b>&gt;(root_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for_transition_capability">apply_for_transition_capability</a>&lt;To: <b>copyable</b>&gt;(root_address: address) {
    <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;To&gt;();
    move_to_sender(<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt; {
        is_certified: <b>false</b>,
        root_address
    });
}
</code></pre>



</details>

<a name="0x0_AccountType_apply_for_granting_capability"></a>

## Function `apply_for_granting_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for_granting_capability">apply_for_granting_capability</a>&lt;To: <b>copyable</b>&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_apply_for_granting_capability">apply_for_granting_capability</a>&lt;To: <b>copyable</b>&gt;() {
    <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;To&gt;();
    move_to_sender(<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt; { is_certified: <b>false</b> });
}
</code></pre>



</details>

<a name="0x0_AccountType_grant_transition_capability"></a>

## Function `grant_transition_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_grant_transition_capability">grant_transition_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_grant_transition_capability">grant_transition_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>, <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> {
    <b>let</b> can_grant = borrow_global&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(Transaction::sender()).is_certified;
    Transaction::assert(can_grant, 2001);
    borrow_global_mut&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(addr).is_certified = <b>true</b>;
}
</code></pre>



</details>

<a name="0x0_AccountType_certify_granting_capability"></a>

## Function `certify_granting_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_certify_granting_capability">certify_granting_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_certify_granting_capability">certify_granting_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    borrow_global_mut&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(addr).is_certified = <b>true</b>;
}
</code></pre>



</details>

<a name="0x0_AccountType_remove_transition_capability"></a>

## Function `remove_transition_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_transition_capability">remove_transition_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_transition_capability">remove_transition_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>, <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> {
    <b>let</b> sender = Transaction::sender();
    <b>let</b> granting_cap = borrow_global&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(sender);
    <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> { is_certified: _, root_address: _ } = move_from&lt;<a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a>&lt;To&gt;&gt;(addr);
    Transaction::assert(granting_cap.is_certified, 2000);
}
</code></pre>



</details>

<a name="0x0_AccountType_remove_granting_capability_from_sender"></a>

## Function `remove_granting_capability_from_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_granting_capability_from_sender">remove_granting_capability_from_sender</a>&lt;To: <b>copyable</b>&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_granting_capability_from_sender">remove_granting_capability_from_sender</a>&lt;To: <b>copyable</b>&gt;()
<b>acquires</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> {
    <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> { is_certified: _ } = move_from&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(Transaction::sender());
}
</code></pre>



</details>

<a name="0x0_AccountType_remove_granting_capability"></a>

## Function `remove_granting_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_granting_capability">remove_granting_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_remove_granting_capability">remove_granting_capability</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a> { is_certified: _ } = move_from&lt;<a href="#0x0_AccountType_GrantingCapability">GrantingCapability</a>&lt;To&gt;&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_AccountType_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_register">register</a>&lt;Type: <b>copyable</b>&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_register">register</a>&lt;Type: <b>copyable</b>&gt;() {
    Transaction::assert(Transaction::sender() == <a href="#0x0_AccountType_singleton_addr">singleton_addr</a>(), 2001);
    move_to_sender(<a href="#0x0_AccountType_Registered">Registered</a>&lt;Type&gt;{});
}
</code></pre>



</details>

<a name="0x0_AccountType_assert_is_a"></a>

## Function `assert_is_a`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;Type: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_assert_is_a">assert_is_a</a>&lt;Type: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_T">T</a> {
    Transaction::assert(<a href="#0x0_AccountType_is_a">is_a</a>&lt;Type&gt;(addr), 2004);
}
</code></pre>



</details>

<a name="0x0_AccountType_assert_has_transition_cap"></a>

## Function `assert_has_transition_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_assert_has_transition_cap">assert_has_transition_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_assert_has_transition_cap">assert_has_transition_cap</a>&lt;To: <b>copyable</b>&gt;(addr: address)
<b>acquires</b> <a href="#0x0_AccountType_TransitionCapability">TransitionCapability</a> {
    Transaction::assert(<a href="#0x0_AccountType_has_transition_cap">has_transition_cap</a>&lt;To&gt;(addr), 2005);
}
</code></pre>



</details>

<a name="0x0_AccountType_singleton_addr"></a>

## Function `singleton_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_singleton_addr">singleton_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountType_singleton_addr">singleton_addr</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x0_AccountType_assert_is_registered"></a>

## Function `assert_is_registered`



<pre><code><b>fun</b> <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;Type: <b>copyable</b>&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountType_assert_is_registered">assert_is_registered</a>&lt;Type: <b>copyable</b>&gt;() {
    Transaction::assert(exists&lt;<a href="#0x0_AccountType_Registered">Registered</a>&lt;Type&gt;&gt;(<a href="#0x0_AccountType_singleton_addr">singleton_addr</a>()), 2003);
}
</code></pre>



</details>
