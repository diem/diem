
<a name="0x1_Event"></a>

# Module `0x1::Event`

### Table of Contents

-  [Struct `EventHandleGenerator`](#0x1_Event_EventHandleGenerator)
-  [Struct `EventHandle`](#0x1_Event_EventHandle)
-  [Function `publish_generator`](#0x1_Event_publish_generator)
-  [Function `fresh_guid`](#0x1_Event_fresh_guid)
-  [Function `new_event_handle`](#0x1_Event_new_event_handle)
-  [Function `emit_event`](#0x1_Event_emit_event)
-  [Function `write_to_event_store`](#0x1_Event_write_to_event_store)
-  [Function `destroy_handle`](#0x1_Event_destroy_handle)
-  [Specification](#0x1_Event_Specification)
    -  [Module specifications](#0x1_Event_@Module_specifications)
        -  [Management of EventHandleGenerators](#0x1_Event_@Management_of_EventHandleGenerators)
        -  [Uniqueness and Counter Incrementation of EventHandleGenerators](#0x1_Event_@Uniqueness_and_Counter_Incrementation_of_EventHandleGenerators)
        -  [Uniqueness of EventHandle GUIDs](#0x1_Event_@Uniqueness_of_EventHandle_GUIDs)
        -  [Destruction of EventHandles](#0x1_Event_@Destruction_of_EventHandles)
    -  [Struct `EventHandleGenerator`](#0x1_Event_Specification_EventHandleGenerator)
    -  [Struct `EventHandle`](#0x1_Event_Specification_EventHandle)
    -  [Function `publish_generator`](#0x1_Event_Specification_publish_generator)
    -  [Function `fresh_guid`](#0x1_Event_Specification_fresh_guid)
    -  [Function `new_event_handle`](#0x1_Event_Specification_new_event_handle)
    -  [Function `emit_event`](#0x1_Event_Specification_emit_event)
    -  [Function `destroy_handle`](#0x1_Event_Specification_destroy_handle)


The Event module defines an
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> that is used to create
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s with unique GUIDs. It contains a counter for the number
of
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s it generates. An
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> is used to count the number of
events emitted to a handle and emit events to the event store.


<a name="0x1_Event_EventHandleGenerator"></a>

## Struct `EventHandleGenerator`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>counter: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>addr: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Event_EventHandle"></a>

## Struct `EventHandle`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>counter: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>guid: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Event_publish_generator"></a>

## Function `publish_generator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_publish_generator">publish_generator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_publish_generator">publish_generator</a>(account: &signer) {
    move_to(account, <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>{ counter: 0, addr: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) })
}
</code></pre>



</details>

<a name="0x1_Event_fresh_guid"></a>

## Function `fresh_guid`



<pre><code><b>fun</b> <a href="#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="#0x1_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <b>let</b> sender_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&counter.addr);
    <b>let</b> count_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&counter.counter);
    counter.counter = counter.counter + 1;

    // <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> goes first just in case we want <b>to</b> extend address in the future.
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> count_bytes, sender_bytes);

    count_bytes
}
</code></pre>



</details>

<a name="0x1_Event_new_event_handle"></a>

## Function `new_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;
<b>acquires</b> <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; {
        counter: 0,
        guid: <a href="#0x1_Event_fresh_guid">fresh_guid</a>(borrow_global_mut&lt;<a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)))
    }
}
</code></pre>



</details>

<a name="0x1_Event_emit_event"></a>

## Function `emit_event`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;, msg: T) {
    <b>let</b> guid = *&handle_ref.guid;

    <a href="#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T&gt;(guid, handle_ref.counter, msg);
    handle_ref.counter = handle_ref.counter + 1;
}
</code></pre>



</details>

<a name="0x1_Event_write_to_event_store"></a>

## Function `write_to_event_store`



<pre><code><b>fun</b> <a href="#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T);
</code></pre>



</details>

<a name="0x1_Event_destroy_handle"></a>

## Function `destroy_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;) {
    <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; { counter: _, guid: _ } = handle;
}
</code></pre>



</details>

<a name="0x1_Event_Specification"></a>

## Specification


<a name="0x1_Event_@Module_specifications"></a>

### Module specifications


Helper function that returns whether or not an EventHandleGenerator is
initilaized at the given address
<code>addr</code>.


<a name="0x1_Event_ehg_exists"></a>


<pre><code><b>define</b> <a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr: address): bool {
    exists&lt;<a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr)
}
</code></pre>


Helper function that returns the EventHandleGenerator at
<code>addr</code>.


<a name="0x1_Event_get_ehg"></a>


<pre><code><b>define</b> <a href="#0x1_Event_get_ehg">get_ehg</a>(addr: address): <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <b>global</b>&lt;<a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr)
}
</code></pre>


Helper function that returns the serialized counter of the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> ehg.


<a name="0x1_Event_serialized_ehg_counter"></a>


<pre><code><b>define</b> <a href="#0x1_Event_serialized_ehg_counter">serialized_ehg_counter</a>(ehg: <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <a href="LCS.md#0x1_LCS_serialize">LCS::serialize</a>(ehg.counter)
}
</code></pre>


Helper function that returns the serialized address of the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> ehg.


<a name="0x1_Event_serialized_ehg_addr"></a>


<pre><code><b>define</b> <a href="#0x1_Event_serialized_ehg_addr">serialized_ehg_addr</a>(ehg: <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <a href="LCS.md#0x1_LCS_serialize">LCS::serialize</a>(ehg.addr)
}
</code></pre>



<a name="0x1_Event_@Management_of_EventHandleGenerators"></a>

#### Management of EventHandleGenerators



<code><a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a></code> is true whenever an
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> is destroyed.
It should never to true to preserve uniqueness of EventHandleGenerators.


<a name="0x1_Event_ehg_destroyed"></a>


<pre><code><b>global</b> <a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a>: bool;
</code></pre>




<a name="0x1_Event_@Uniqueness_and_Counter_Incrementation_of_EventHandleGenerators"></a>

#### Uniqueness and Counter Incrementation of EventHandleGenerators



<a name="0x1_Event_EHGCounterUnchanged"></a>

**Informally:** If an
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> exists, then it never changes
except when a function generates a new
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> GUID.


<pre><code><b>schema</b> <a href="#0x1_Event_EHGCounterUnchanged">EHGCounterUnchanged</a> {
    <b>ensures</b> forall addr: address where <b>old</b>(<a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr)):
                <a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr) && <a href="#0x1_Event_get_ehg">get_ehg</a>(addr).counter == <b>old</b>(<a href="#0x1_Event_get_ehg">get_ehg</a>(addr).counter);
}
</code></pre>




<a name="0x1_Event_EHGCounterIncreasesOnEventHandleCreate"></a>

**Informally:** If the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> has been initialized,
then the counter never decreases and stays initialized.
This proves the uniqueness of the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>s at entry and
exit of functions in this module.

> TODO(#4549): Unable to prove thaat the counter increments using schemas.
However, we can prove the that the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> increments
in the post conditions of the functions.

Base case:


<pre><code><b>schema</b> <a href="#0x1_Event_EHGCounterIncreasesOnEventHandleCreate">EHGCounterIncreasesOnEventHandleCreate</a> {
    <b>ensures</b> forall addr: address where !<b>old</b>(<a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr)) && <a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr):
                <a href="#0x1_Event_get_ehg">get_ehg</a>(addr).counter == 0;
}
</code></pre>


Induction step:

> TODO(kkmc): Change
<code>counter &gt;= <b>old</b>(counter)</code> to
<code>counter == <b>old</b>(counter) + 1</code>
Currently, this can be proved at the function level but not the global invariant level.


<pre><code><b>schema</b> <a href="#0x1_Event_EHGCounterIncreasesOnEventHandleCreate">EHGCounterIncreasesOnEventHandleCreate</a> {
    <b>ensures</b> forall addr: address where <b>old</b>(<a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr)):
                <a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr) && <a href="#0x1_Event_get_ehg">get_ehg</a>(addr).counter &gt;= <b>old</b>(<a href="#0x1_Event_get_ehg">get_ehg</a>(addr).counter);
}
</code></pre>



Apply
<code><a href="#0x1_Event_EHGCounterUnchanged">EHGCounterUnchanged</a></code> to all functions except for those
that create a new GUID and increment the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> counter.


<pre><code><b>apply</b> <a href="#0x1_Event_EHGCounterUnchanged">EHGCounterUnchanged</a> <b>to</b> * <b>except</b> fresh_guid, new_event_handle;
</code></pre>


Apply
<code><a href="#0x1_Event_EHGCounterIncreasesOnEventHandleCreate">EHGCounterIncreasesOnEventHandleCreate</a></code> to all functions except fresh_guid
and its callees.


<pre><code><b>apply</b> <a href="#0x1_Event_EHGCounterIncreasesOnEventHandleCreate">EHGCounterIncreasesOnEventHandleCreate</a> <b>to</b> *;
</code></pre>




<a name="0x1_Event_@Uniqueness_of_EventHandle_GUIDs"></a>

#### Uniqueness of EventHandle GUIDs



<a name="0x1_Event_UniqueEventHandleGUIDs"></a>

**Informally:** All
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s have unqiue GUIDs.

There are several ways we may want to express this invariant:

INV 1: The first invariant is that all
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s have unique GUIDs.
An intuitive way to write this is to keep a list of previously existing
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s
and for every
<code><b>pack</b></code> of an
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>, the GUID must be different from all the
previously existing ones. However, this requires us to keep track of all of the
newly generated
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> resources (possibly through a ghost variable) and
compare their GUIDs to each other, which is currently not possible.

INV 2: Enforce
<code>fresh_guid</code> to only return unique GUIDs; every call only increases the counter
of the EventHandleGenerator and the output of the
<code>fresh_guid</code>.

> TODO(kkmc): The move-prover does not currently encode the property that LCS::serialize
returns the same number of bytes for the same primitive types. Which means that
<code>fresh_guid</code> can return the same GUIDs.

E.g. If LCS::serialize(addr1) == <0,1,2>, LCS::serialize(addr2) == <0,1>,
LCS::serialize(ctr1) == <3>, LCS::serialize(ctr2) == <2,3>,
then <0,1,2> appended with <3> is equal to <0,1> appended with <2,3>.


<pre><code><b>schema</b> <a href="#0x1_Event_UniqueEventHandleGUIDs">UniqueEventHandleGUIDs</a> {
    <b>invariant</b> <b>module</b> <b>true</b>;
}
</code></pre>



Apply
<code><a href="#0x1_Event_UniqueEventHandleGUIDs">UniqueEventHandleGUIDs</a></code> to enforce all
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> resources to have unique GUIDs.


<pre><code><b>apply</b> <a href="#0x1_Event_UniqueEventHandleGUIDs">UniqueEventHandleGUIDs</a> <b>to</b> *;
</code></pre>





<a name="0x1_Event_@Destruction_of_EventHandles"></a>

#### Destruction of EventHandles


Variable that counts the total number of event handles ever to exist.


<a name="0x1_Event_total_num_of_event_handles"></a>


<pre><code><b>global</b> <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt;: num;
</code></pre>



<a name="0x1_Event_Specification_EventHandleGenerator"></a>

### Struct `EventHandleGenerator`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>
</code></pre>



<dl>
<dt>

<code>counter: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>addr: address</code>
</dt>
<dd>

</dd>
</dl>


Updates the ehg_destroyed variable to true if an
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> is ever unpacked.


<pre><code><b>invariant</b> <b>pack</b> <a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a> = <a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a>;
<b>invariant</b> <b>unpack</b> <a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a> = <b>true</b>;
</code></pre>




<a name="0x1_Event_EventHandleGeneratorNeverDestroyed"></a>

**Informally:** No
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> should ever be destroyed.
Together with
<code><a href="#0x1_Event_EventHandleGeneratorAtSameAddress">EventHandleGeneratorAtSameAddress</a></code>, the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>s
can never be alternated.


<pre><code><b>schema</b> <a href="#0x1_Event_EventHandleGeneratorNeverDestroyed">EventHandleGeneratorNeverDestroyed</a> {
    <b>invariant</b> <b>module</b> !<a href="#0x1_Event_ehg_destroyed">ehg_destroyed</a>;
}
</code></pre>




<a name="0x1_Event_EventHandleGeneratorAtSameAddress"></a>

**Informally:** An EventHandleGenerator should be located at
<code>addr</code> and is never moved.


<pre><code><b>schema</b> <a href="#0x1_Event_EventHandleGeneratorAtSameAddress">EventHandleGeneratorAtSameAddress</a> {
    <b>invariant</b> <b>module</b> forall addr: address where <a href="#0x1_Event_ehg_exists">ehg_exists</a>(addr): <a href="#0x1_Event_get_ehg">get_ehg</a>(addr).addr == addr;
}
</code></pre>



Apply
<code><a href="#0x1_Event_EventHandleGeneratorNeverDestroyed">EventHandleGeneratorNeverDestroyed</a></code> to all functions to ensure that the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>s are never destroyed.
Together with
<code><a href="#0x1_Event_EHGCounterIncreasesOnEventHandleCreate">EHGCounterIncreasesOnEventHandleCreate</a></code>, this proves
the uniqueness of the
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> resource throughout transient
states at each address.
Without this, the specification would allow an implementation to remove
an
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>, say
<code>ehg</code>, and then have it generate a number
of events until the
<code>ehg.counter</code> >=
<code><b>old</b>(ehg.counter)</code>. Violating the property
that an EventHandleGenerator should only be used to generate unique GUIDs.

> TODO(#4549): Potential bug. Without
<code>* <b>except</b> fresh_guid;</code>, this
takes a long time to return from the Boogie solver. Sometimes it
returns a precondition violation error quickly (seemingly
non-determistic). We expect all functions to satisfy this schema.


<pre><code><b>apply</b> <a href="#0x1_Event_EventHandleGeneratorNeverDestroyed">EventHandleGeneratorNeverDestroyed</a> <b>to</b> * <b>except</b> fresh_guid;
</code></pre>


Apply
<code><a href="#0x1_Event_EventHandleGeneratorAtSameAddress">EventHandleGeneratorAtSameAddress</a></code> to all functions to enforce
that all
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>s reside at the address they hold.

> TODO(#4549): Potential bug. Refer to the previous TODO.
The solver takes a long time unless fresh_guid is excepted.


<pre><code><b>apply</b> <a href="#0x1_Event_EventHandleGeneratorAtSameAddress">EventHandleGeneratorAtSameAddress</a> <b>to</b> * <b>except</b> fresh_guid;
</code></pre>



<a name="0x1_Event_Specification_EventHandle"></a>

### Struct `EventHandle`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T: <b>copyable</b>&gt;
</code></pre>



<dl>
<dt>

<code>counter: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>guid: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


Count the total number of
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s.
This is used in the post condition of
<code>destroy_handle</code> to ensure that an
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> is destroyed.


<pre><code><b>invariant</b> <b>pack</b> <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt; = <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt; + 1;
<b>invariant</b> <b>unpack</b> <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt; = <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt; - 1;
</code></pre>



<a name="0x1_Event_Specification_publish_generator"></a>

### Function `publish_generator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_publish_generator">publish_generator</a>(account: &signer)
</code></pre>



Creates a new
<code><a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> with an initial counter 0 and the
signer
<code>account</code>'s address.


<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account))
            == <a href="#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> { counter: 0, addr: <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account) };
</code></pre>



<a name="0x1_Event_Specification_fresh_guid"></a>

### Function `fresh_guid`


<pre><code><b>fun</b> <a href="#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="#0x1_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): vector&lt;u8&gt;
</code></pre>



The byte array returned is the concatenation of the serialized
EventHandleGenerator counter and address.


<pre><code><b>aborts_if</b> counter.counter + 1 &gt; max_u64();
<b>ensures</b> counter.counter == <b>old</b>(counter).counter + 1;
<b>ensures</b> <a href="Vector.md#0x1_Vector_eq_append">Vector::eq_append</a>(
            result,
            <b>old</b>(<a href="#0x1_Event_serialized_ehg_counter">serialized_ehg_counter</a>(counter)),
            <b>old</b>(<a href="#0x1_Event_serialized_ehg_addr">serialized_ehg_addr</a>(counter))
        );
</code></pre>



<a name="0x1_Event_Specification_new_event_handle"></a>

### Function `new_event_handle`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Event_ehg_exists">ehg_exists</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account));
<b>aborts_if</b> <a href="#0x1_Event_get_ehg">get_ehg</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account)).counter + 1 &gt; max_u64();
<b>ensures</b> <a href="#0x1_Event_ehg_exists">ehg_exists</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account));
<b>ensures</b> <a href="#0x1_Event_get_ehg">get_ehg</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account)).counter ==
            <b>old</b>(<a href="#0x1_Event_get_ehg">get_ehg</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account)).counter) + 1;
<b>ensures</b> result.counter == 0;
<b>ensures</b> <a href="Vector.md#0x1_Vector_eq_append">Vector::eq_append</a>(
            result.guid,
            <b>old</b>(<a href="#0x1_Event_serialized_ehg_counter">serialized_ehg_counter</a>(<a href="#0x1_Event_get_ehg">get_ehg</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account)))),
            <b>old</b>(<a href="#0x1_Event_serialized_ehg_addr">serialized_ehg_addr</a>(<a href="#0x1_Event_get_ehg">get_ehg</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account))))
        );
</code></pre>



<a name="0x1_Event_Specification_emit_event"></a>

### Function `emit_event`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;, msg: T)
</code></pre>



The counter in
<code><a href="#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;</code> increases and the event is emitted to the event store.

> TODO(kkmc): Do we need to specify that the event was sent to the event store?


<pre><code><b>aborts_if</b> handle_ref.counter + 1 &gt; max_u64();
<b>ensures</b> handle_ref.counter == <b>old</b>(handle_ref.counter) + 1;
<b>ensures</b> handle_ref.guid == <b>old</b>(handle_ref.guid);
</code></pre>



<a name="0x1_Event_Specification_destroy_handle"></a>

### Function `destroy_handle`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;)
</code></pre>




<code>destroy_handle</code> should have unpacked an
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code> and thereby decreasing the
total number of
<code><a href="#0x1_Event_EventHandle">EventHandle</a></code>s.


<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt; == <b>old</b>(<a href="#0x1_Event_total_num_of_event_handles">total_num_of_event_handles</a>&lt;T&gt;) - 1;
</code></pre>
