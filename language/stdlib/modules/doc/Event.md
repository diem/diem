
<a name="0x1_Event"></a>

# Module `0x1::Event`

The Event module defines an <code><a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> that is used to create
<code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code>s with unique GUIDs. It contains a counter for the number
of <code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code>s it generates. An <code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code> is used to count the number of
events emitted to a handle and emit events to the event store.


-  [Resource `EventHandleGenerator`](#0x1_Event_EventHandleGenerator)
-  [Resource `EventHandle`](#0x1_Event_EventHandle)
-  [Constants](#@Constants_0)
-  [Function `publish_generator`](#0x1_Event_publish_generator)
-  [Function `fresh_guid`](#0x1_Event_fresh_guid)
-  [Function `new_event_handle`](#0x1_Event_new_event_handle)
-  [Function `emit_event`](#0x1_Event_emit_event)
-  [Function `write_to_event_store`](#0x1_Event_write_to_event_store)
-  [Function `destroy_handle`](#0x1_Event_destroy_handle)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="BCS.md#0x1_BCS">0x1::BCS</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Event_EventHandleGenerator"></a>

## Resource `EventHandleGenerator`

A resource representing the counter used to generate uniqueness under each account. There won't be destructor for
this resource to guarantee the uniqueness of the generated handle.


<pre><code><b>resource</b> <b>struct</b> <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>
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

## Resource `EventHandle`

A handle for an event such that:
1. Other modules can emit events to this handle.
2. Storage can use this handle to prove the total number of events that happened in the past.


<pre><code><b>resource</b> <b>struct</b> <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>counter: u64</code>
</dt>
<dd>
 Total number of events emitted to this event stream.
</dd>
<dt>
<code>guid: vector&lt;u8&gt;</code>
</dt>
<dd>
 A globally unique ID for this event stream.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Event_EEVENT_GENERATOR"></a>

The event generator resource was in an invalid state


<pre><code><b>const</b> <a href="Event.md#0x1_Event_EEVENT_GENERATOR">EEVENT_GENERATOR</a>: u64 = 0;
</code></pre>



<a name="0x1_Event_publish_generator"></a>

## Function `publish_generator`

Publishs a new event handle generator.


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_publish_generator">publish_generator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_publish_generator">publish_generator</a>(account: &signer) {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="Event.md#0x1_Event_EEVENT_GENERATOR">EEVENT_GENERATOR</a>));
    move_to(account, <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>{ counter: 0, addr })
}
</code></pre>



</details>

<a name="0x1_Event_fresh_guid"></a>

## Function `fresh_guid`

Derive a fresh unique id by using sender's EventHandleGenerator. The generated vector<u8> is indeed unique because it
was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
such counter is going to give distinct value for each of the new event stream under each sender. And since we
hash it with the sender's address, the result is guaranteed to be globally unique.


<pre><code><b>fun</b> <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <b>let</b> sender_bytes = <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(&counter.addr);
    <b>let</b> count_bytes = <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(&counter.counter);
    counter.counter = counter.counter + 1;

    // <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> goes first just in case we want <b>to</b> extend address in the future.
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> count_bytes, sender_bytes);

    count_bytes
}
</code></pre>



</details>

<a name="0x1_Event_new_event_handle"></a>

## Function `new_event_handle`

Use EventHandleGenerator to generate a unique event handle for <code>sig</code>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;
<b>acquires</b> <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<b>exists</b>&lt;<a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Event.md#0x1_Event_EEVENT_GENERATOR">EEVENT_GENERATOR</a>));
    <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; {
        counter: 0,
        guid: <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(borrow_global_mut&lt;<a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr))
    }
}
</code></pre>



</details>

<a name="0x1_Event_emit_event"></a>

## Function `emit_event`

Emit an event with payload <code>msg</code> by using <code>handle_ref</code>'s key and counter.


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;, msg: T) {
    <b>let</b> guid = *&handle_ref.guid;

    <a href="Event.md#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T&gt;(guid, handle_ref.counter, msg);
    handle_ref.counter = handle_ref.counter + 1;
}
</code></pre>



</details>

<a name="0x1_Event_write_to_event_store"></a>

## Function `write_to_event_store`

Native procedure that writes to the actual event stream in Event store
This will replace the "native" portion of EmitEvent bytecode


<pre><code><b>fun</b> <a href="Event.md#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Event.md#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T);
</code></pre>



</details>

<a name="0x1_Event_destroy_handle"></a>

## Function `destroy_handle`

Destroy a unique handle.


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;) {
    <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; { counter: _, guid: _ } = handle;
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



> NOTE: specification and verification of event related functionality is currently not happening.
> Since events cannot be observed from Move programs, this does not affect the verification of
> other functionality; however, this should be completed at a later point to ensure the framework
> generates events as expected.

Functions of the event module are mocked out using the intrinsic
pragma. They are implemented in the prover's prelude as no-ops.


<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions
