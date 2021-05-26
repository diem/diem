
<a name="0x1_Vault"></a>

# Module `0x1::Vault`

A module which implements secure memory (called a *vault*) of some content which can only be operated
on if authorized by a signer. Authorization is managed by
[*capabilities*](https://en.wikipedia.org/wiki/Capability-based_security). The vault supports delegation
of capabilities to other signers (including revocation) as well as transfer of ownership.


<a name="@Overview_0"></a>

## Overview



<a name="@Capabilities_1"></a>

### Capabilities


Capabilities are unforgeable tokens which represent the right to perform a particular
operation on the vault. To acquire a capability instance, authentication via a signer is needed.
This signer must either be the owner of the vault, or someone the capability has been delegated to.
Once acquired, a capability can be passed to other functions to perform the operation it enables.
Specifically, those called functions do not need to have access to the original signer. This is a key
property of capability based security as it prevents granting more rights to code than needed.

Capability instances are unforgeable because they are localized to transactions. They can only be
created by functions of this module, and as they do not have the Move language <code>store</code> or <code>key</code> abilities,
they cannot leak out of a transaction.

Example:

```move
struct Content has store { ssn: u64 }
...
// Create new vault
Vault::new(signer, b"My Vault", Content{ ssn: 525659745 });
...
// Obtain a read capability
let read_cap = Vault::acquire_read_cap<Content>(signer);
process(&read_cap)
...
fun process(cap: &Vault::ReadCap<Content>) {
let accessor = Vault::read_accessor(cap);
let content = Vault::borrow(accessor);
<< do something with <code>content: &Content</code> >>
Vault::release_read_accessor(accessor);
}
```


<a name="@Delegation_2"></a>

### Delegation


Delegation provides the option to delegate the right to acquire a vault capability to other
signers than the owner of the vault. Delegates still need to authenticate themselves using their
signer, similar as the owner does. All security arguments for owners apply also to delegates.
Delegation can be revoked removing previously granted rights from a delegate.

Delegation can be configured to be transitive by granting the right to acquire a delegation capability
to delegates, which can then further delegate access rights.

By default, when a vault is created, it does not support delegation. The owner of the vault
needs to explicitly enable delegation. This allows to create vaults which are not intended for delegation
and one does not need to worry about its misuse.

Example:

```move
Vault::new(signer, b"My Vault", Content{ ssn: 525659745 });
// Enable delegation for this vault. Only the owning signer can do this.
Vault::enable_delegation<Content>(signer);
...
// Delegate read capability to some other signer.
let delegate_cap = Vault::acquire_delegate_cap<Content>(signer);
Vault::delegate_read_cap(&delegate_cap, other_signer);
...
// Other signer can now acquire read cap
let read_cap = Vault::acquire_read_cap<Content>(other_signer);
...
// The granted capability can be revoked. There is no need to have the other signer for this.
Vault::revoke_read_cap(&delegate_cap, Signer::address_of(other_signer));
```


<a name="@Abilities_3"></a>

### Abilities


Currently, we require that the <code>Content</code> type of a vault has the <code>drop</code> ability in order to instantiate
a capability type like <code><a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>&lt;Content&gt;</code>. Without this, capabilities themselves would need to have an
explicit release function, which makes little sense as they are pure values. We expect the Move
language to have 'phantom type parameters' or similar features added, which will allows us to have
<code><a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>&lt;Content&gt;</code> droppable and copyable without <code>Content</code> needing the same.


-  [Overview](#@Overview_0)
    -  [Capabilities](#@Capabilities_1)
    -  [Delegation](#@Delegation_2)
    -  [Abilities](#@Abilities_3)
-  [Struct `ReadCap`](#0x1_Vault_ReadCap)
-  [Struct `ModifyCap`](#0x1_Vault_ModifyCap)
-  [Struct `DelegateCap`](#0x1_Vault_DelegateCap)
-  [Struct `TransferCap`](#0x1_Vault_TransferCap)
-  [Struct `CapType`](#0x1_Vault_CapType)
-  [Struct `VaultDelegateEvent`](#0x1_Vault_VaultDelegateEvent)
-  [Struct `VaultTransferEvent`](#0x1_Vault_VaultTransferEvent)
-  [Resource `Vault`](#0x1_Vault_Vault)
-  [Resource `VaultDelegates`](#0x1_Vault_VaultDelegates)
-  [Resource `VaultEvents`](#0x1_Vault_VaultEvents)
-  [Resource `VaultDelegate`](#0x1_Vault_VaultDelegate)
-  [Struct `ReadAccessor`](#0x1_Vault_ReadAccessor)
-  [Struct `ModifyAccessor`](#0x1_Vault_ModifyAccessor)
-  [Constants](#@Constants_4)
-  [Function `read_cap_type`](#0x1_Vault_read_cap_type)
-  [Function `modify_cap_type`](#0x1_Vault_modify_cap_type)
-  [Function `delegate_cap_type`](#0x1_Vault_delegate_cap_type)
-  [Function `transfer_cap_type`](#0x1_Vault_transfer_cap_type)
-  [Function `new`](#0x1_Vault_new)
-  [Function `is_delegation_enabled`](#0x1_Vault_is_delegation_enabled)
-  [Function `enable_delegation`](#0x1_Vault_enable_delegation)
-  [Function `enable_events`](#0x1_Vault_enable_events)
-  [Function `remove_vault`](#0x1_Vault_remove_vault)
-  [Function `acquire_read_cap`](#0x1_Vault_acquire_read_cap)
-  [Function `acquire_modify_cap`](#0x1_Vault_acquire_modify_cap)
-  [Function `acquire_delegate_cap`](#0x1_Vault_acquire_delegate_cap)
-  [Function `acquire_transfer_cap`](#0x1_Vault_acquire_transfer_cap)
-  [Function `validate_cap`](#0x1_Vault_validate_cap)
-  [Function `read_accessor`](#0x1_Vault_read_accessor)
-  [Function `borrow`](#0x1_Vault_borrow)
-  [Function `release_read_accessor`](#0x1_Vault_release_read_accessor)
-  [Function `modify_accessor`](#0x1_Vault_modify_accessor)
-  [Function `borrow_mut`](#0x1_Vault_borrow_mut)
-  [Function `release_modify_accessor`](#0x1_Vault_release_modify_accessor)
-  [Function `delegate`](#0x1_Vault_delegate)
-  [Function `revoke`](#0x1_Vault_revoke)
-  [Function `revoke_all`](#0x1_Vault_revoke_all)
-  [Function `remove_element`](#0x1_Vault_remove_element)
-  [Function `add_element`](#0x1_Vault_add_element)
-  [Function `emit_delegate_event`](#0x1_Vault_emit_delegate_event)
-  [Function `transfer`](#0x1_Vault_transfer)


<pre><code><b>use</b> <a href="">0x1::Errors</a>;
<b>use</b> <a href="">0x1::Event</a>;
<b>use</b> <a href="">0x1::Option</a>;
<b>use</b> <a href="">0x1::Signer</a>;
<b>use</b> <a href="">0x1::Vector</a>;
</code></pre>



<a name="0x1_Vault_ReadCap"></a>

## Struct `ReadCap`

A capability to read the content of the vault. Notice that the capability cannot be
stored but can be freely copied and dropped.
TODO: remove <code>drop</code> on <code>Content</code> here and elsewhere once we have phantom type parameters.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>&lt;Content: drop, store&gt; has <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_ModifyCap"></a>

## Struct `ModifyCap`

A capability to modify the content of the vault.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_ModifyCap">ModifyCap</a>&lt;Content: drop, store&gt; has <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_DelegateCap"></a>

## Struct `DelegateCap`

A capability to delegate access to the vault.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content: drop, store&gt; has <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_TransferCap"></a>

## Struct `TransferCap`

A capability to transfer ownership of the vault.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_TransferCap">TransferCap</a>&lt;Content: drop, store&gt; has <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_CapType"></a>

## Struct `CapType`

A type describing a capability. This is used for functions like <code><a href="Vault.md#0x1_Vault_delegate">Self::delegate</a></code> where we need to
specify capability types.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_CapType">CapType</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>code: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_VaultDelegateEvent"></a>

## Struct `VaultDelegateEvent`

An event which we generate on vault access delegation or revocation if event generation is enabled.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_VaultDelegateEvent">VaultDelegateEvent</a> has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
<dt>
<code>delegate: address</code>
</dt>
<dd>

</dd>
<dt>
<code>cap: <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a></code>
</dt>
<dd>

</dd>
<dt>
<code>is_revoked: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_VaultTransferEvent"></a>

## Struct `VaultTransferEvent`

An event which we generate on vault transfer if event generation is enabled.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_VaultTransferEvent">VaultTransferEvent</a> has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>authority: address</code>
</dt>
<dd>

</dd>
<dt>
<code>new_vault_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_Vault"></a>

## Resource `Vault`

Private. The vault representation.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault">Vault</a>&lt;Content: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>content: <a href="_Option">Option::Option</a>&lt;Content&gt;</code>
</dt>
<dd>
 The content. If the option is empty, the content is currently moved into an
 accessor in order to work with it.
</dd>
</dl>


</details>

<a name="0x1_Vault_VaultDelegates"></a>

## Resource `VaultDelegates`

Private. If the vault supports delegation, information about the delegates.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>delegates: vector&lt;address&gt;</code>
</dt>
<dd>
 The currently authorized delegates.
</dd>
</dl>


</details>

<a name="0x1_Vault_VaultEvents"></a>

## Resource `VaultEvents`

Private. If event generation is enabled, contains the event generators.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>
 Metadata which identifies this vault. This information is used
 in events generated by this module.
</dd>
<dt>
<code>delegate_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="Vault.md#0x1_Vault_VaultDelegateEvent">Vault::VaultDelegateEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for vault delegation.
</dd>
<dt>
<code>transfer_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="Vault.md#0x1_Vault_VaultTransferEvent">Vault::VaultTransferEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for vault transfer.
</dd>
</dl>


</details>

<a name="0x1_Vault_VaultDelegate"></a>

## Resource `VaultDelegate`

Private. A value stored at a delegates address pointing to the owner of the vault. Also
describes the capabilities granted to this delegate.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content: store&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>granted_caps: vector&lt;<a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_ReadAccessor"></a>

## Struct `ReadAccessor`

A read accessor for the content of the vault.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>&lt;Content: drop, store&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>content: Content</code>
</dt>
<dd>

</dd>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Vault_ModifyAccessor"></a>

## Struct `ModifyAccessor`

A modify accessor for the content of the vault.


<pre><code><b>struct</b> <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>&lt;Content: drop, store&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>content: Content</code>
</dt>
<dd>

</dd>
<dt>
<code>vault_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_4"></a>

## Constants


<a name="0x1_Vault_EACCESSOR_INCONSISTENCY"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EACCESSOR_INCONSISTENCY">EACCESSOR_INCONSISTENCY</a>: u64 = 3;
</code></pre>



<a name="0x1_Vault_EACCESSOR_IN_USE"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EACCESSOR_IN_USE">EACCESSOR_IN_USE</a>: u64 = 2;
</code></pre>



<a name="0x1_Vault_EDELEGATE"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EDELEGATE">EDELEGATE</a>: u64 = 1;
</code></pre>



<a name="0x1_Vault_EDELEGATE_TO_SELF"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EDELEGATE_TO_SELF">EDELEGATE_TO_SELF</a>: u64 = 4;
</code></pre>



<a name="0x1_Vault_EDELEGATION_NOT_ENABLED"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EDELEGATION_NOT_ENABLED">EDELEGATION_NOT_ENABLED</a>: u64 = 5;
</code></pre>



<a name="0x1_Vault_EEVENT"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EEVENT">EEVENT</a>: u64 = 6;
</code></pre>



<a name="0x1_Vault_EVAULT"></a>



<pre><code><b>const</b> <a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>: u64 = 0;
</code></pre>



<a name="0x1_Vault_read_cap_type"></a>

## Function `read_cap_type`

Creates a read capability type.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_read_cap_type">read_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_read_cap_type">read_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">CapType</a> { <a href="Vault.md#0x1_Vault_CapType">CapType</a>{ code : 0 } }
</code></pre>



</details>

<a name="0x1_Vault_modify_cap_type"></a>

## Function `modify_cap_type`

Creates a modify  capability type.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_modify_cap_type">modify_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_modify_cap_type">modify_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">CapType</a> { <a href="Vault.md#0x1_Vault_CapType">CapType</a>{ code : 1 } }
</code></pre>



</details>

<a name="0x1_Vault_delegate_cap_type"></a>

## Function `delegate_cap_type`

Creates a delegate  capability type.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_delegate_cap_type">delegate_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_delegate_cap_type">delegate_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">CapType</a> { <a href="Vault.md#0x1_Vault_CapType">CapType</a>{ code : 2 } }
</code></pre>



</details>

<a name="0x1_Vault_transfer_cap_type"></a>

## Function `transfer_cap_type`

Creates a transfer  capability type.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_transfer_cap_type">transfer_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_transfer_cap_type">transfer_cap_type</a>(): <a href="Vault.md#0x1_Vault_CapType">CapType</a> { <a href="Vault.md#0x1_Vault_CapType">CapType</a>{ code : 3 } }
</code></pre>



</details>

<a name="0x1_Vault_new"></a>

## Function `new`

Creates new vault for the given signer. The vault is populated with the <code>initial_content</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_new">new</a>&lt;Content: store&gt;(owner: &signer, initial_content: Content)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_new">new</a>&lt;Content: store&gt;(owner: &signer,  initial_content: Content) {
    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(owner);
    <b>assert</b>(!<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr), <a href="_already_published">Errors::already_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
    move_to&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(
        owner,
        <a href="Vault.md#0x1_Vault">Vault</a>{
            content: <a href="_some">Option::some</a>(initial_content)
        }
    )
}
</code></pre>



</details>

<a name="0x1_Vault_is_delegation_enabled"></a>

## Function `is_delegation_enabled`

Returns <code><b>true</b></code> if the delegation functionality has been enabled.
Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_is_delegation_enabled">is_delegation_enabled</a>&lt;Content: store&gt;(owner: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_is_delegation_enabled">is_delegation_enabled</a>&lt;Content: store&gt;(owner: &signer): bool {
    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(owner);
    <b>assert</b>(<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr), <a href="_not_published">Errors::not_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
    <b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Vault_enable_delegation"></a>

## Function `enable_delegation`

Enables delegation functionality for this vault. By default, vaults to not support delegation.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_enable_delegation">enable_delegation</a>&lt;Content: store&gt;(owner: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_enable_delegation">enable_delegation</a>&lt;Content: store&gt;(owner: &signer) {
    <b>assert</b>(!<a href="Vault.md#0x1_Vault_is_delegation_enabled">is_delegation_enabled</a>&lt;Content&gt;(owner), <a href="_already_published">Errors::already_published</a>(<a href="Vault.md#0x1_Vault_EDELEGATE">EDELEGATE</a>));
    move_to&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(owner, <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>{delegates: <a href="_empty">Vector::empty</a>()})
}
</code></pre>



</details>

<a name="0x1_Vault_enable_events"></a>

## Function `enable_events`

Enables event generation for this vault. This passed metadata is used to identify
the vault in events.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_enable_events">enable_events</a>&lt;Content: store&gt;(owner: &signer, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_enable_events">enable_events</a>&lt;Content: store&gt;(owner: &signer, metadata: vector&lt;u8&gt;) {
    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(owner);
    <b>assert</b>(<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr), <a href="_not_published">Errors::not_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
    <b>assert</b>(!<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(addr), <a href="_already_published">Errors::already_published</a>(<a href="Vault.md#0x1_Vault_EEVENT">EEVENT</a>));
    move_to&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(
        owner,
        <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>{
            metadata,
            delegate_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="Vault.md#0x1_Vault_VaultDelegateEvent">VaultDelegateEvent</a>&gt;(owner),
            transfer_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="Vault.md#0x1_Vault_VaultTransferEvent">VaultTransferEvent</a>&gt;(owner),
        }
    );
}
</code></pre>



</details>

<a name="0x1_Vault_remove_vault"></a>

## Function `remove_vault`

Removes a vault and all its associated data, returning the current content. In order for
this to succeed, there must be no active accessor for the vault.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_remove_vault">remove_vault</a>&lt;Content: drop, store&gt;(owner: &signer): Content
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_remove_vault">remove_vault</a>&lt;Content: store + drop&gt;(owner: &signer): Content
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a>, <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>, <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>, <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a> {
    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(owner);
    <b>assert</b>(<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr), <a href="_not_published">Errors::not_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
    <b>let</b> <a href="Vault.md#0x1_Vault">Vault</a>{content} = move_from&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr);
    <b>assert</b>(<a href="_is_some">Option::is_some</a>(&content), <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_IN_USE">EACCESSOR_IN_USE</a>));

    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(addr)) {
        <b>let</b> delegate_cap = <a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;{vault_address: addr, authority: addr};
        <a href="Vault.md#0x1_Vault_revoke_all">revoke_all</a>(&delegate_cap);
    };
    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(addr)) {
        <b>let</b> <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>{metadata: _metadata, delegate_events, transfer_events} =
            move_from&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(addr);
        <a href="_destroy_handle">Event::destroy_handle</a>(delegate_events);
        <a href="_destroy_handle">Event::destroy_handle</a>(transfer_events);
    };

    <a href="_extract">Option::extract</a>(&<b>mut</b> content)
}
</code></pre>



</details>

<a name="0x1_Vault_acquire_read_cap"></a>

## Function `acquire_read_cap`

Acquires the capability to read the vault. The passed signer must either be the owner
of the vault or a delegate with appropriate access.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_read_cap">acquire_read_cap</a>&lt;Content: drop, store&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_ReadCap">Vault::ReadCap</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_read_cap">acquire_read_cap</a>&lt;Content: store + drop&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> {
    <b>let</b> (vault_address, authority) = <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content&gt;(requester, <a href="Vault.md#0x1_Vault_read_cap_type">read_cap_type</a>());
    <a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>{ vault_address, authority }
}
</code></pre>



</details>

<a name="0x1_Vault_acquire_modify_cap"></a>

## Function `acquire_modify_cap`

Acquires the capability to modify the vault. The passed signer must either be the owner
of the vault or a delegate with appropriate access.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_modify_cap">acquire_modify_cap</a>&lt;Content: drop, store&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_ModifyCap">Vault::ModifyCap</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_modify_cap">acquire_modify_cap</a>&lt;Content: store + drop&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_ModifyCap">ModifyCap</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> {
    <b>let</b> (vault_address, authority) = <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content&gt;(requester, <a href="Vault.md#0x1_Vault_modify_cap_type">modify_cap_type</a>());
    <a href="Vault.md#0x1_Vault_ModifyCap">ModifyCap</a>{ vault_address, authority }
}
</code></pre>



</details>

<a name="0x1_Vault_acquire_delegate_cap"></a>

## Function `acquire_delegate_cap`

Acquires the capability to delegate access to the vault. The passed signer must either be the owner
of the vault or a delegate with appropriate access.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_delegate_cap">acquire_delegate_cap</a>&lt;Content: drop, store&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_DelegateCap">Vault::DelegateCap</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_delegate_cap">acquire_delegate_cap</a>&lt;Content: store + drop&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> {
    <b>let</b> (vault_address, authority) = <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content&gt;(requester, <a href="Vault.md#0x1_Vault_delegate_cap_type">delegate_cap_type</a>());
    <a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>{ vault_address, authority }
}
</code></pre>



</details>

<a name="0x1_Vault_acquire_transfer_cap"></a>

## Function `acquire_transfer_cap`

Acquires the capability to transfer the vault. The passed signer must either be the owner
of the vault or a delegate with appropriate access.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_transfer_cap">acquire_transfer_cap</a>&lt;Content: drop, store&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_TransferCap">Vault::TransferCap</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_acquire_transfer_cap">acquire_transfer_cap</a>&lt;Content: store + drop&gt;(requester: &signer): <a href="Vault.md#0x1_Vault_TransferCap">TransferCap</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> {
    <b>let</b> (vault_address, authority) = <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content&gt;(requester, <a href="Vault.md#0x1_Vault_transfer_cap_type">transfer_cap_type</a>());
    <a href="Vault.md#0x1_Vault_TransferCap">TransferCap</a>{ vault_address, authority }
}
</code></pre>



</details>

<a name="0x1_Vault_validate_cap"></a>

## Function `validate_cap`

Private. Validates whether a capability can be acquired by the given signer. Returns the
pair of the vault address and the used authority.


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content: drop, store&gt;(requester: &signer, cap: <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>): (address, address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_validate_cap">validate_cap</a>&lt;Content: store + drop&gt;(requester: &signer, cap: <a href="Vault.md#0x1_Vault_CapType">CapType</a>): (address, address)
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> {
    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(requester);
    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr)) {
        // The signer is a delegate. Check it's granted capabilities.
        <b>let</b> delegate = borrow_global&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr);
        <b>assert</b>(<a href="_contains">Vector::contains</a>(&delegate.granted_caps, &cap), <a href="_requires_capability">Errors::requires_capability</a>(<a href="Vault.md#0x1_Vault_EDELEGATE">EDELEGATE</a>));
        (delegate.vault_address, addr)
    } <b>else</b> {
        // If it is not a delegate, it must be the owner <b>to</b> succeed.
        <b>assert</b>(<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(addr), <a href="_not_published">Errors::not_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
        (addr, addr)
    }
}
</code></pre>



</details>

<a name="0x1_Vault_read_accessor"></a>

## Function `read_accessor`

Creates a read accessor for the content in the vault based on a read capability.

Only one accessor (whether read or modify) for the same vault can exist at a time, and this
function will abort if one is in use. An accessor must be explicitly released using
<code><a href="Vault.md#0x1_Vault_release_read_accessor">Self::release_read_accessor</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_read_accessor">read_accessor</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_ReadCap">Vault::ReadCap</a>&lt;Content&gt;): <a href="Vault.md#0x1_Vault_ReadAccessor">Vault::ReadAccessor</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_read_accessor">read_accessor</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_ReadCap">ReadCap</a>&lt;Content&gt;): <a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a> {
    <b>let</b> content = &<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(cap.vault_address).content;
    <b>assert</b>(<a href="_is_some">Option::is_some</a>(content), <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_IN_USE">EACCESSOR_IN_USE</a>));
    <a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>{ vault_address: cap.vault_address, content: <a href="_extract">Option::extract</a>(content) }
}
</code></pre>



</details>

<a name="0x1_Vault_borrow"></a>

## Function `borrow`

Returns a reference to the content represented by a read accessor.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_borrow">borrow</a>&lt;Content: drop, store&gt;(accessor: &<a href="Vault.md#0x1_Vault_ReadAccessor">Vault::ReadAccessor</a>&lt;Content&gt;): &Content
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_borrow">borrow</a>&lt;Content: store + drop&gt;(accessor: &<a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>&lt;Content&gt;): &Content {
    &accessor.content
}
</code></pre>



</details>

<a name="0x1_Vault_release_read_accessor"></a>

## Function `release_read_accessor`

Releases read accessor.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_release_read_accessor">release_read_accessor</a>&lt;Content: drop, store&gt;(accessor: <a href="Vault.md#0x1_Vault_ReadAccessor">Vault::ReadAccessor</a>&lt;Content&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_release_read_accessor">release_read_accessor</a>&lt;Content: store + drop&gt;(accessor: <a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>&lt;Content&gt;)
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a> {
    <b>let</b> <a href="Vault.md#0x1_Vault_ReadAccessor">ReadAccessor</a>{ content: new_content, vault_address } = accessor;
    <b>let</b> content = &<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(vault_address).content;
    // We (should be/are) able <b>to</b> prove that the below cannot happen, but we leave the assertion
    // here anyway for double safety.
    <b>assert</b>(<a href="_is_none">Option::is_none</a>(content), <a href="_internal">Errors::internal</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_INCONSISTENCY">EACCESSOR_INCONSISTENCY</a>));
    <a href="_fill">Option::fill</a>(content, new_content);
}
</code></pre>



</details>

<a name="0x1_Vault_modify_accessor"></a>

## Function `modify_accessor`

Creates a modify accessor for the content in the vault based on a modify capability. This
is similar like <code><a href="Vault.md#0x1_Vault_read_accessor">Self::read_accessor</a></code> but the returned accessor will allow to mutate
the content.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_modify_accessor">modify_accessor</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_ModifyCap">Vault::ModifyCap</a>&lt;Content&gt;): <a href="Vault.md#0x1_Vault_ModifyAccessor">Vault::ModifyAccessor</a>&lt;Content&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_modify_accessor">modify_accessor</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_ModifyCap">ModifyCap</a>&lt;Content&gt;): <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>&lt;Content&gt;
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a> {
    <b>let</b> content = &<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(cap.vault_address).content;
    <b>assert</b>(<a href="_is_some">Option::is_some</a>(content), <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_IN_USE">EACCESSOR_IN_USE</a>));
    <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>{ vault_address: cap.vault_address, content: <a href="_extract">Option::extract</a>(content) }
}
</code></pre>



</details>

<a name="0x1_Vault_borrow_mut"></a>

## Function `borrow_mut`

Returns a mutable reference to the content represented by a modify accessor.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_borrow_mut">borrow_mut</a>&lt;Content: drop, store&gt;(accessor: &<b>mut</b> <a href="Vault.md#0x1_Vault_ModifyAccessor">Vault::ModifyAccessor</a>&lt;Content&gt;): &<b>mut</b> Content
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_borrow_mut">borrow_mut</a>&lt;Content: store + drop&gt;(accessor: &<b>mut</b> <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>&lt;Content&gt;): &<b>mut</b> Content {
    &<b>mut</b> accessor.content
}
</code></pre>



</details>

<a name="0x1_Vault_release_modify_accessor"></a>

## Function `release_modify_accessor`

Releases a modify accessor. This will ensure that any modifications are written back
to the vault.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_release_modify_accessor">release_modify_accessor</a>&lt;Content: drop, store&gt;(accessor: <a href="Vault.md#0x1_Vault_ModifyAccessor">Vault::ModifyAccessor</a>&lt;Content&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_release_modify_accessor">release_modify_accessor</a>&lt;Content: store + drop&gt;(accessor: <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>&lt;Content&gt;)
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a> {
    <b>let</b> <a href="Vault.md#0x1_Vault_ModifyAccessor">ModifyAccessor</a>{ content: new_content, vault_address } = accessor;
    <b>let</b> content = &<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(vault_address).content;
    // We (should be/are) able <b>to</b> prove that the below cannot happen, but we leave the assertion
    // here anyway for double safety.
    <b>assert</b>(<a href="_is_none">Option::is_none</a>(content), <a href="_internal">Errors::internal</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_INCONSISTENCY">EACCESSOR_INCONSISTENCY</a>));
    <a href="_fill">Option::fill</a>(content, new_content);
}
</code></pre>



</details>

<a name="0x1_Vault_delegate"></a>

## Function `delegate`

Delegates the right to acquire a capability of the given type. Delegation must have been enabled
during vault creation for this to succeed.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_delegate">delegate</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">Vault::DelegateCap</a>&lt;Content&gt;, to_signer: &signer, cap_type: <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_delegate">delegate</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;, to_signer: &signer, cap_type: <a href="Vault.md#0x1_Vault_CapType">CapType</a>)
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>, <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>, <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a> {
    <b>assert</b>(
        <b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address),
        <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EDELEGATION_NOT_ENABLED">EDELEGATION_NOT_ENABLED</a>)
    );

    <b>let</b> addr = <a href="_address_of">Signer::address_of</a>(to_signer);
    <b>assert</b>(addr != cap.vault_address, <a href="_invalid_argument">Errors::invalid_argument</a>(<a href="Vault.md#0x1_Vault_EDELEGATE_TO_SELF">EDELEGATE_TO_SELF</a>));

    <b>if</b> (!<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr)) {
        // Create <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a> <b>if</b> it is not yet existing.
        move_to&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(
            to_signer,
            <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>{vault_address: cap.vault_address, granted_caps: <a href="_empty">Vector::empty</a>()}
        );
        // Add the the delegate <b>to</b> <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>.
        <b>let</b> vault_delegates = borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address);
        <a href="Vault.md#0x1_Vault_add_element">add_element</a>(&<b>mut</b> vault_delegates.delegates, addr);
    };

    // Grant the capability.
    <b>let</b> delegate = borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr);
    <a href="Vault.md#0x1_Vault_add_element">add_element</a>(&<b>mut</b> delegate.granted_caps, *&cap_type);

    // Generate event
    <a href="Vault.md#0x1_Vault_emit_delegate_event">emit_delegate_event</a>(cap, cap_type, addr, <b>false</b>);
}
</code></pre>



</details>

<a name="0x1_Vault_revoke"></a>

## Function `revoke`

Revokes the delegated right to acquire a capability of given type.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_revoke">revoke</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">Vault::DelegateCap</a>&lt;Content&gt;, addr: address, cap_type: <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_revoke">revoke</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;, addr: address, cap_type: <a href="Vault.md#0x1_Vault_CapType">CapType</a>)
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>, <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>, <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a> {
    <b>assert</b>(
        <b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address),
        <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EDELEGATION_NOT_ENABLED">EDELEGATION_NOT_ENABLED</a>)
    );
    <b>assert</b>(<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr), <a href="_not_published">Errors::not_published</a>(<a href="Vault.md#0x1_Vault_EDELEGATE">EDELEGATE</a>));

    <b>let</b> delegate = borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr);
    <a href="Vault.md#0x1_Vault_remove_element">remove_element</a>(&<b>mut</b> delegate.granted_caps, &cap_type);

    // If the granted caps of this delegate drop <b>to</b> zero, remove it.
    <b>if</b> (<a href="_is_empty">Vector::is_empty</a>(&delegate.granted_caps)) {
        <b>let</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>{ vault_address: _owner, granted_caps: _granted_caps} =
            move_from&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(addr);
        <b>let</b> vault_delegates = borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address);
        <a href="Vault.md#0x1_Vault_remove_element">remove_element</a>(&<b>mut</b> vault_delegates.delegates, &addr);
    };

    // Generate event.
    <a href="Vault.md#0x1_Vault_emit_delegate_event">emit_delegate_event</a>(cap, cap_type, addr, <b>true</b>);
}
</code></pre>



</details>

<a name="0x1_Vault_revoke_all"></a>

## Function `revoke_all`

Revokes all delegate rights for this vault.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_revoke_all">revoke_all</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">Vault::DelegateCap</a>&lt;Content&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_revoke_all">revoke_all</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;)
<b>acquires</b> <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>, <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>, <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a> {
    <b>assert</b>(
        <b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address),
        <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EDELEGATION_NOT_ENABLED">EDELEGATION_NOT_ENABLED</a>)
    );
    <b>let</b> delegates = &<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address).delegates;
    <b>while</b> (!<a href="_is_empty">Vector::is_empty</a>(delegates)) {
        <b>let</b> addr = <a href="_pop_back">Vector::pop_back</a>(delegates);
        <b>let</b> <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>{ vault_address: _vault_address, granted_caps} =
            move_from&lt;<a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>&lt;Content&gt;&gt;(cap.vault_address);
        <b>while</b> (!<a href="_is_empty">Vector::is_empty</a>(&granted_caps)) {
            <b>let</b> cap_type = <a href="_pop_back">Vector::pop_back</a>(&<b>mut</b> granted_caps);
            <a href="Vault.md#0x1_Vault_emit_delegate_event">emit_delegate_event</a>(cap, cap_type, addr, <b>true</b>);
        }
    }
}
</code></pre>



</details>

<a name="0x1_Vault_remove_element"></a>

## Function `remove_element`

Helper to remove an element from a vector.


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_remove_element">remove_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: &E)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_remove_element">remove_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: &E) {
    <b>let</b> (found, index) = <a href="_index_of">Vector::index_of</a>(v, x);
    <b>if</b> (found) {
        <a href="_remove">Vector::remove</a>(v, index);
    }
}
</code></pre>



</details>

<a name="0x1_Vault_add_element"></a>

## Function `add_element`

Helper to add an element to a vector.


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_add_element">add_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: E)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_add_element">add_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: E) {
    <b>if</b> (!<a href="_contains">Vector::contains</a>(v, &x)) {
        <a href="_push_back">Vector::push_back</a>(v, x)
    }
}
</code></pre>



</details>

<a name="0x1_Vault_emit_delegate_event"></a>

## Function `emit_delegate_event`

Emits a delegation or revocation event if event generation is enabled.


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_emit_delegate_event">emit_delegate_event</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_DelegateCap">Vault::DelegateCap</a>&lt;Content&gt;, cap_type: <a href="Vault.md#0x1_Vault_CapType">Vault::CapType</a>, delegate: address, is_revoked: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Vault.md#0x1_Vault_emit_delegate_event">emit_delegate_event</a>&lt;Content: store + drop&gt;(
       cap: &<a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;,
       cap_type: <a href="Vault.md#0x1_Vault_CapType">CapType</a>,
       delegate: address,
       is_revoked: bool
) <b>acquires</b> <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address)) {
        <b>let</b> event = <a href="Vault.md#0x1_Vault_VaultDelegateEvent">VaultDelegateEvent</a>{
            metadata: *&borrow_global&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address).metadata,
            vault_address: cap.vault_address,
            authority: cap.authority,
            delegate,
            cap: cap_type,
            is_revoked
        };
        <a href="_emit_event">Event::emit_event</a>(&<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address).delegate_events, event);
    }
}
</code></pre>



</details>

<a name="0x1_Vault_transfer"></a>

## Function `transfer`

Transfers ownership of the vault to a new signer. All delegations are revoked before transfer,
and the new owner must re-create delegates as needed.


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_transfer">transfer</a>&lt;Content: drop, store&gt;(cap: &<a href="Vault.md#0x1_Vault_TransferCap">Vault::TransferCap</a>&lt;Content&gt;, to_owner: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vault.md#0x1_Vault_transfer">transfer</a>&lt;Content: store + drop&gt;(cap: &<a href="Vault.md#0x1_Vault_TransferCap">TransferCap</a>&lt;Content&gt;, to_owner: &signer)
<b>acquires</b> <a href="Vault.md#0x1_Vault">Vault</a>, <a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>, <a href="Vault.md#0x1_Vault_VaultDelegate">VaultDelegate</a>, <a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a> {
    <b>let</b> new_addr = <a href="_address_of">Signer::address_of</a>(to_owner);
    <b>assert</b>(!<b>exists</b>&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(new_addr), <a href="_already_published">Errors::already_published</a>(<a href="Vault.md#0x1_Vault_EVAULT">EVAULT</a>));
    <b>assert</b>(
        <a href="_is_some">Option::is_some</a>(&borrow_global&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(cap.vault_address).content),
        <a href="_invalid_state">Errors::invalid_state</a>(<a href="Vault.md#0x1_Vault_EACCESSOR_IN_USE">EACCESSOR_IN_USE</a>)
    );

    // Revoke all delegates.
    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultDelegates">VaultDelegates</a>&lt;Content&gt;&gt;(cap.vault_address)) {
        <b>let</b> delegate_cap = <a href="Vault.md#0x1_Vault_DelegateCap">DelegateCap</a>&lt;Content&gt;{vault_address: cap.vault_address, authority: cap.authority };
        <a href="Vault.md#0x1_Vault_revoke_all">revoke_all</a>(&delegate_cap);
    };

    // Emit event <b>if</b> event generation is enabled. We emit the event on the <b>old</b> vault not the new one.
    <b>if</b> (<b>exists</b>&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address)) {
        <b>let</b> event = <a href="Vault.md#0x1_Vault_VaultTransferEvent">VaultTransferEvent</a> {
            metadata: *&borrow_global&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address).metadata,
            vault_address: cap.vault_address,
            authority: cap.authority,
            new_vault_address: new_addr
        };
        <a href="_emit_event">Event::emit_event</a>(&<b>mut</b> borrow_global_mut&lt;<a href="Vault.md#0x1_Vault_VaultEvents">VaultEvents</a>&lt;Content&gt;&gt;(cap.vault_address).transfer_events, event);
    };

    // Move the vault.
    move_to&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(to_owner, move_from&lt;<a href="Vault.md#0x1_Vault">Vault</a>&lt;Content&gt;&gt;(cap.vault_address));
}
</code></pre>



</details>
