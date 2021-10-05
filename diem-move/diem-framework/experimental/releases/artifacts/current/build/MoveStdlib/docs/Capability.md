
<a name="0x1_Capability"></a>

# Module `0x1::Capability`

A module which defines the basic concept of
[*capabilities*](https://en.wikipedia.org/wiki/Capability-based_security) for managing access control.

EXPERIMENTAL


<a name="@Overview_0"></a>

## Overview


A capability is a unforgeable token which testifies that a signer has authorized a certain operation.
The token is valid during the transaction where it is obtained. Since the type <code><a href="Capability.md#0x1_Capability_Cap">Capability::Cap</a></code> has
no ability to be stored in global memory, capabilities cannot leak out of a transaction. For every function
called within a transaction which has a capability as a parameter, it is guaranteed that the capability
has been obtained via a proper signer-based authorization step previously in the transaction's execution.


<a name="@Usage_1"></a>

### Usage


Initializing and acquiring capabilities is usually encapsulated in a module with a type
tag which can only be constructed by this module.

```
module Pkg::Feature {
use Std::Capability::Cap;

/// A type tag used in Cap<Feature>. Only this module can create an instance,
/// and there is no public function other than Self::acquire which returns a value of this type.
/// This way, this module has full control how Cap<Feature> is given out.
struct Feature has drop {}

/// Initializes this module.
public fun initialize(s: &signer) {
// Create capability. This happens once at module initialization time.
// One needs to provide a witness for being the owner of Feature
// in the 2nd parameter.
<<additional conditions allowing to initialize this capability>>
Capability::create<Feature>(s, &Feature{});
}

/// Acquires the capability to work with this feature.
public fun acquire(s: &signer): Cap<Feature> {
<<additional conditions allowing to acquire this capability>>
Capability::acquire<Feature>(s, &Feature{});
}

/// Does something related to the feature. The caller must pass a Cap<Feature>.
public fun do_something(_cap: Cap<Feature>) { ... }
}
```


<a name="@Delegation_2"></a>

### Delegation


Capabilities come with the optional feature of *delegation*. Via <code><a href="Capability.md#0x1_Capability_delegate">Self::delegate</a></code>, an owner of a capability
can designate another signer to be also capable of acquiring the capability. Like the original creator,
the delegate needs to present his signer to obtain the capability in his transactions. Delegation can
be revoked via <code><a href="Capability.md#0x1_Capability_revoke">Self::revoke</a></code>, removing this access right from the delegate.

While the basic authorization mechanism for delegates is the same as with core capabilities, the
target of delegation might be subject of restrictions which need to be specified and verified. This can
be done via global invariants in the specification language. For example, in order to prevent delegation
all together for a capability, one can use the following invariant:

```
invariant forall a: address where Capability::spec_has_cap<Feature>(a):
len(Capability::spec_delegates<Feature>(a)) == 0;
```

Similarly, the following invariant would enforce that delegates, if existent, must satisfy a certain
predicate:

```
invariant forall a: address where Capability::spec_has_cap<Feature>(a):
forall d in Capability::spec_delegates<Feature>(a):
is_valid_delegate_for_feature(d);
```


-  [Overview](#@Overview_0)
    -  [Usage](#@Usage_1)
    -  [Delegation](#@Delegation_2)
-  [Struct `Cap`](#0x1_Capability_Cap)
-  [Struct `LinearCap`](#0x1_Capability_LinearCap)
-  [Resource `CapState`](#0x1_Capability_CapState)
-  [Resource `CapDelegateState`](#0x1_Capability_CapDelegateState)
-  [Constants](#@Constants_3)
-  [Function `create`](#0x1_Capability_create)
-  [Function `acquire`](#0x1_Capability_acquire)
-  [Function `acquire_linear`](#0x1_Capability_acquire_linear)
-  [Function `validate_acquire`](#0x1_Capability_validate_acquire)
-  [Function `root_addr`](#0x1_Capability_root_addr)
-  [Function `linear_root_addr`](#0x1_Capability_linear_root_addr)
-  [Function `delegate`](#0x1_Capability_delegate)
-  [Function `revoke`](#0x1_Capability_revoke)
-  [Function `remove_element`](#0x1_Capability_remove_element)
-  [Function `add_element`](#0x1_Capability_add_element)
-  [Module Specification](#@Module_Specification_4)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Capability_Cap"></a>

## Struct `Cap`

The token representing an acquired capability. Cannot be stored in memory, but copied and dropped freely.


<pre><code><b>struct</b> <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt; has <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>root: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Capability_LinearCap"></a>

## Struct `LinearCap`

A linear version of a capability token. This can be used if an acquired capability should be enforced
to be used only once for an authorization.


<pre><code><b>struct</b> <a href="Capability.md#0x1_Capability_LinearCap">LinearCap</a>&lt;Feature&gt; has drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>root: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Capability_CapState"></a>

## Resource `CapState`

An internal data structure for representing a configured capability.


<pre><code><b>struct</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>delegates: vector&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Capability_CapDelegateState"></a>

## Resource `CapDelegateState`

An internal data structure for representing a configured delegated capability.


<pre><code><b>struct</b> <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt; has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>root: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_3"></a>

## Constants


<a name="0x1_Capability_ECAP"></a>



<pre><code><b>const</b> <a href="Capability.md#0x1_Capability_ECAP">ECAP</a>: u64 = 0;
</code></pre>



<a name="0x1_Capability_EDELEGATE"></a>



<pre><code><b>const</b> <a href="Capability.md#0x1_Capability_EDELEGATE">EDELEGATE</a>: u64 = 1;
</code></pre>



<a name="0x1_Capability_create"></a>

## Function `create`

Creates a new capability class, owned by the passed signer. A caller must pass a witness that
they own the <code>Feature</code> type parameter.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_create">create</a>&lt;Feature&gt;(owner: &signer, _feature_witness: &Feature)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_create">create</a>&lt;Feature&gt;(owner: &signer, _feature_witness: &Feature) {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(owner);
    <b>assert</b>!(!<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="Capability.md#0x1_Capability_ECAP">ECAP</a>));
    move_to&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(owner, <a href="Capability.md#0x1_Capability_CapState">CapState</a>{ delegates: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>

<a name="0x1_Capability_acquire"></a>

## Function `acquire`

Acquires a capability token. Only the owner of the capability class, or an authorized delegate,
can succeed with this operation. A caller must pass a witness that they own the <code>Feature</code> type
parameter.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_acquire">acquire</a>&lt;Feature&gt;(requester: &signer, _feature_witness: &Feature): <a href="Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;Feature&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_acquire">acquire</a>&lt;Feature&gt;(requester: &signer, _feature_witness: &Feature): <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt;
<b>acquires</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a>, <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a> {
    <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt;{root: <a href="Capability.md#0x1_Capability_validate_acquire">validate_acquire</a>&lt;Feature&gt;(requester)}
}
</code></pre>



</details>

<a name="0x1_Capability_acquire_linear"></a>

## Function `acquire_linear`

Acquires a linear capability token. It is up to the module which owns <code>Feature</code> to decide
whether to expose a linear or non-linear capability.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_acquire_linear">acquire_linear</a>&lt;Feature&gt;(requester: &signer, _feature_witness: &Feature): <a href="Capability.md#0x1_Capability_LinearCap">Capability::LinearCap</a>&lt;Feature&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_acquire_linear">acquire_linear</a>&lt;Feature&gt;(requester: &signer, _feature_witness: &Feature): <a href="Capability.md#0x1_Capability_LinearCap">LinearCap</a>&lt;Feature&gt;
<b>acquires</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a>, <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a> {
    <a href="Capability.md#0x1_Capability_LinearCap">LinearCap</a>&lt;Feature&gt;{root: <a href="Capability.md#0x1_Capability_validate_acquire">validate_acquire</a>&lt;Feature&gt;(requester)}
}
</code></pre>



</details>

<a name="0x1_Capability_validate_acquire"></a>

## Function `validate_acquire`

Helper to validate an acquire. Returns the root address of the capability.


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_validate_acquire">validate_acquire</a>&lt;Feature&gt;(requester: &signer): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_validate_acquire">validate_acquire</a>&lt;Feature&gt;(requester: &signer): address
<b>acquires</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a>, <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(requester);
    <b>if</b> (<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;&gt;(addr)) {
        <b>let</b> root_addr = borrow_global&lt;<a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;&gt;(addr).root;
        // double check that requester is actually registered <b>as</b> a delegate
        <b>assert</b>!(<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(root_addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Capability.md#0x1_Capability_EDELEGATE">EDELEGATE</a>));
        <b>assert</b>!(<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&borrow_global&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(root_addr).delegates, &addr),
               <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Capability.md#0x1_Capability_EDELEGATE">EDELEGATE</a>));
        root_addr
    } <b>else</b> {
        <b>assert</b>!(<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Capability.md#0x1_Capability_ECAP">ECAP</a>));
        addr
    }
}
</code></pre>



</details>

<a name="0x1_Capability_root_addr"></a>

## Function `root_addr`

Returns the root address associated with the given capability token. Only the owner
of the feature can do this.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_root_addr">root_addr</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;Feature&gt;, _feature_witness: &Feature): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_root_addr">root_addr</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt;, _feature_witness: &Feature): address {
    cap.root
}
</code></pre>



</details>

<a name="0x1_Capability_linear_root_addr"></a>

## Function `linear_root_addr`

Returns the root address associated with the given linear capability token.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_linear_root_addr">linear_root_addr</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_LinearCap">Capability::LinearCap</a>&lt;Feature&gt;, _feature_witness: &Feature): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_linear_root_addr">linear_root_addr</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_LinearCap">LinearCap</a>&lt;Feature&gt;, _feature_witness: &Feature): address {
    cap.root
}
</code></pre>



</details>

<a name="0x1_Capability_delegate"></a>

## Function `delegate`

Registers a delegation relation. If the relation already exists, this function does
nothing.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_delegate">delegate</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;Feature&gt;, _feature_witness: &Feature, <b>to</b>: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_delegate">delegate</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt;, _feature_witness: &Feature, <b>to</b>: &signer)
<b>acquires</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(<b>to</b>);
    <b>if</b> (<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;&gt;(addr)) <b>return</b>;
    move_to(<b>to</b>, <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;{root: cap.root});
    <a href="Capability.md#0x1_Capability_add_element">add_element</a>(&<b>mut</b> borrow_global_mut&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(cap.root).delegates, addr);
}
</code></pre>



</details>

<a name="0x1_Capability_revoke"></a>

## Function `revoke`

Revokes a delegation relation. If no relation exists, this function does nothing.


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_revoke">revoke</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;Feature&gt;, _feature_witness: &Feature, from: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Capability.md#0x1_Capability_revoke">revoke</a>&lt;Feature&gt;(cap: <a href="Capability.md#0x1_Capability_Cap">Cap</a>&lt;Feature&gt;, _feature_witness: &Feature, from: address)
<b>acquires</b> <a href="Capability.md#0x1_Capability_CapState">CapState</a>, <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>
{
    <b>if</b> (!<b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;&gt;(from)) <b>return</b>;
    <b>let</b> <a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>{root: _root} = move_from&lt;<a href="Capability.md#0x1_Capability_CapDelegateState">CapDelegateState</a>&lt;Feature&gt;&gt;(from);
    <a href="Capability.md#0x1_Capability_remove_element">remove_element</a>(&<b>mut</b> borrow_global_mut&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(cap.root).delegates, &from);
}
</code></pre>



</details>

<a name="0x1_Capability_remove_element"></a>

## Function `remove_element`

Helper to remove an element from a vector.


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_remove_element">remove_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: &E)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_remove_element">remove_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: &E) {
    <b>let</b> (found, index) = <a href="Vector.md#0x1_Vector_index_of">Vector::index_of</a>(v, x);
    <b>if</b> (found) {
        <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>(v, index);
    }
}
</code></pre>



</details>

<a name="0x1_Capability_add_element"></a>

## Function `add_element`

Helper to add an element to a vector.


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_add_element">add_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: E)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_add_element">add_element</a>&lt;E: drop&gt;(v: &<b>mut</b> vector&lt;E&gt;, x: E) {
    <b>if</b> (!<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(v, &x)) {
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(v, x)
    }
}
</code></pre>



</details>

<a name="@Module_Specification_4"></a>

## Module Specification

Helper specification function to check whether a capability exists at address.


<a name="0x1_Capability_spec_has_cap"></a>


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_spec_has_cap">spec_has_cap</a>&lt;Feature&gt;(addr: address): bool {
   <b>exists</b>&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(addr)
}
</code></pre>


Helper specification function to obtain the delegates of a capability.


<a name="0x1_Capability_spec_delegates"></a>


<pre><code><b>fun</b> <a href="Capability.md#0x1_Capability_spec_delegates">spec_delegates</a>&lt;Feature&gt;(addr: address): vector&lt;address&gt; {
   <b>global</b>&lt;<a href="Capability.md#0x1_Capability_CapState">CapState</a>&lt;Feature&gt;&gt;(addr).delegates
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
