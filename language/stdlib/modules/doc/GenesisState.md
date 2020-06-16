
<a name="0x1_GenesisState"></a>

# Module `0x1::GenesisState`

### Table of Contents

-  [Struct `GenesisState`](#0x1_GenesisState_GenesisState)
-  [Function `genesis_address`](#0x1_GenesisState_genesis_address)
-  [Function `begin_genesis`](#0x1_GenesisState_begin_genesis)
-  [Function `end_genesis`](#0x1_GenesisState_end_genesis)
-  [Function `is_during_genesis`](#0x1_GenesisState_is_during_genesis)
-  [Function `is_after_genesis`](#0x1_GenesisState_is_after_genesis)
-  [Specification](#0x1_GenesisState_Specification)
    -  [GenesisState](#0x1_GenesisState_@GenesisState)
        -  [GenesisState state machine](#0x1_GenesisState_@GenesisState_state_machine)
    -  [Function `genesis_address`](#0x1_GenesisState_Specification_genesis_address)
    -  [Function `begin_genesis`](#0x1_GenesisState_Specification_begin_genesis)
    -  [Function `end_genesis`](#0x1_GenesisState_Specification_end_genesis)

This file implements a state variable to track the status of genesis.
It is in a separate leave module, because it will be used by lots of
other modules to check whether functions are being called during the
genesis process or not. Putting it in Genesis would introduce a ton
of dependencies, but using GenesisState will not.


<a name="0x1_GenesisState_GenesisState"></a>

## Struct `GenesisState`

A singleton resource that stores the state of the genesis process
on genesis_address()


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_GenesisState">GenesisState</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>genesis_complete: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_GenesisState_genesis_address"></a>

## Function `genesis_address`

**TODO:** May want to move to CoreAddresses, or just use
ASSOCIATION_ROOT_ADDRESS


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_genesis_address">genesis_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_genesis_address">genesis_address</a>(): address {
    <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()
}
</code></pre>



</details>

<a name="0x1_GenesisState_begin_genesis"></a>

## Function `begin_genesis`

Abort if genesis has already begun, to protect against unverified
code re-starting genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_begin_genesis">begin_genesis</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_begin_genesis">begin_genesis</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x1_GenesisState_genesis_address">genesis_address</a>(), 1942);
    move_to(account, <a href="#0x1_GenesisState">GenesisState</a> { genesis_complete: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_GenesisState_end_genesis"></a>

## Function `end_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_end_genesis">end_genesis</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_end_genesis">end_genesis</a>() <b>acquires</b> <a href="#0x1_GenesisState">GenesisState</a> {
    <b>let</b> gen_state = borrow_global_mut&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_genesis_address">genesis_address</a>());
    gen_state.genesis_complete = <b>true</b>;
}
</code></pre>



</details>

<a name="0x1_GenesisState_is_during_genesis"></a>

## Function `is_during_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_is_during_genesis">is_during_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_is_during_genesis">is_during_genesis</a>(): bool <b>acquires</b> <a href="#0x1_GenesisState">GenesisState</a> {
    <b>let</b> gen_state = borrow_global_mut&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_genesis_address">genesis_address</a>());
    gen_state.genesis_complete == <b>false</b>
}
</code></pre>



</details>

<a name="0x1_GenesisState_is_after_genesis"></a>

## Function `is_after_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_is_after_genesis">is_after_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_is_after_genesis">is_after_genesis</a>(): bool <b>acquires</b> <a href="#0x1_GenesisState">GenesisState</a> {
    <b>let</b> gen_state = borrow_global_mut&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_genesis_address">genesis_address</a>());
    gen_state.genesis_complete == <b>true</b>
}
</code></pre>



</details>

<a name="0x1_GenesisState_Specification"></a>

## Specification


<a name="0x1_GenesisState_@GenesisState"></a>

### GenesisState



<a name="0x1_GenesisState_@GenesisState_state_machine"></a>

#### GenesisState state machine


There is a state machine that tracks the genesis process.  There are three states,
and it must proceed through them in strict sequences.
**before_genesis:** Before start of genesis, there are no instances of the resource,
and, of course, none stored at the genesis_address.
**during_genesis:** the resource is stored at genesis_address and the "genesis_complete"
field is "false".
**after_genesis:** When genesis completes, the resource is stored at genesis_address
and the "genesis_complete" field is true.

For the security and correctness of the system, there are no other transitions.
Hence, the transitions from state before_genesis -> during_genesis and
transition from state 2 -> 3 are???
irreversible.  Correctness of initialization in many modules depends on
initialization not changing the state after the first call, so it is important
that no code be able to cause the system to enter the genesis states out-of-order.

Helper functions
Instead of literal address, should refer to CoreAddresses.


<a name="0x1_GenesisState_spec_genesis_address"></a>


<pre><code><b>define</b> <a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>(): address {
    0xA550C18
}
<a name="0x1_GenesisState_spec_genesis_state"></a>
<b>define</b> <a href="#0x1_GenesisState_spec_genesis_state">spec_genesis_state</a>(): <a href="#0x1_GenesisState">GenesisState</a> {
    <b>global</b>&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>())
}
<a name="0x1_GenesisState_spec_is_before_genesis"></a>
<b>define</b> <a href="#0x1_GenesisState_spec_is_before_genesis">spec_is_before_genesis</a>(): bool {
    !exists&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>())
}
<a name="0x1_GenesisState_spec_is_during_genesis"></a>
<b>define</b> <a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>(): bool {
    exists&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>())
    && !<b>global</b>&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>()).genesis_complete
}
<a name="0x1_GenesisState_spec_is_after_genesis"></a>
<b>define</b> <a href="#0x1_GenesisState_spec_is_after_genesis">spec_is_after_genesis</a>(): bool {
    exists&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>())
    && <b>global</b>&lt;<a href="#0x1_GenesisState">GenesisState</a>&gt;(<a href="#0x1_GenesisState_spec_genesis_address">spec_genesis_address</a>()).genesis_complete
}
</code></pre>




<a name="0x1_GenesisState_TransBeginToDuring"></a>

**Informally:** If current state is before genesis, the very next state
must be during genesis.
Note that this does not allow the system to remain "before genesis" for
even one function call, so the transition must happen in the first function
call.


<pre><code><b>schema</b> <a href="#0x1_GenesisState_TransBeginToDuring">TransBeginToDuring</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_GenesisState_spec_is_before_genesis">spec_is_before_genesis</a>()) ==&gt; <a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>();
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_GenesisState_TransBeginToDuring">TransBeginToDuring</a> <b>to</b> * <b>except</b> genesis_address;
</code></pre>




<a name="0x1_GenesisState_TransBeginToAfter"></a>

**Informally:** If the system has started but not finished genesis,
the next state will either be the same, or it will end genesis.


<pre><code><b>schema</b> <a href="#0x1_GenesisState_TransBeginToAfter">TransBeginToAfter</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>())
            ==&gt; (<a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>() || <a href="#0x1_GenesisState_spec_is_after_genesis">spec_is_after_genesis</a>());
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_GenesisState_TransBeginToAfter">TransBeginToAfter</a> <b>to</b> *;
</code></pre>




<a name="0x1_GenesisState_AfterPersists"></a>

**Informally:** If the system has started but not finished genesis,
the next state will either be the same, or it will end genesis.


<pre><code><b>schema</b> <a href="#0x1_GenesisState_AfterPersists">AfterPersists</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_GenesisState_spec_is_after_genesis">spec_is_after_genesis</a>()) ==&gt; <a href="#0x1_GenesisState_spec_is_after_genesis">spec_is_after_genesis</a>();
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_GenesisState_AfterPersists">AfterPersists</a> <b>to</b> *;
</code></pre>



<a name="0x1_GenesisState_Specification_genesis_address"></a>

### Function `genesis_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_genesis_address">genesis_address</a>(): address
</code></pre>



Does not change genesis state. This is because of "except genesis_address"
in the apply of TransBeginToDuring.


<pre><code><b>ensures</b> <b>old</b>(<a href="#0x1_GenesisState_spec_genesis_state">spec_genesis_state</a>()) == <a href="#0x1_GenesisState_spec_genesis_state">spec_genesis_state</a>();
</code></pre>



<a name="0x1_GenesisState_Specification_begin_genesis"></a>

### Function `begin_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_begin_genesis">begin_genesis</a>(account: &signer)
</code></pre>




<pre><code><b>ensures</b> <a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>();
</code></pre>



<a name="0x1_GenesisState_Specification_end_genesis"></a>

### Function `end_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_GenesisState_end_genesis">end_genesis</a>()
</code></pre>



Requires that end_genesis only be called during genesis, which
means that it can only be called once. There is no great harm if
it is called multiple times, but reporting it as an error might
catch code bugs.


<pre><code><b>requires</b> <a href="#0x1_GenesisState_spec_is_during_genesis">spec_is_during_genesis</a>();
<b>ensures</b> <a href="#0x1_GenesisState_spec_is_after_genesis">spec_is_after_genesis</a>();
</code></pre>
