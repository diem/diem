
<a name="0x1_Event"></a>

# Module `0x1::Event`


The Event module defines an <code><a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code> that is used to create
<code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code>s with unique GUIDs. It contains a counter for the number
of <code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code>s it generates. An <code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code> is used to count the number of
events emitted to a handle and emit events to the event store.


-  [Resource <code><a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a></code>](#0x1_Event_EventHandleGenerator)
-  [Resource <code><a href="Event.md#0x1_Event_EventHandle">EventHandle</a></code>](#0x1_Event_EventHandle)
-  [Function <code>publish_generator</code>](#0x1_Event_publish_generator)
-  [Function <code>fresh_guid</code>](#0x1_Event_fresh_guid)
-  [Function <code>new_event_handle</code>](#0x1_Event_new_event_handle)
-  [Function <code>emit_event</code>](#0x1_Event_emit_event)
-  [Function <code>write_to_event_store</code>](#0x1_Event_write_to_event_store)
-  [Function <code>destroy_handle</code>](#0x1_Event_destroy_handle)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_Event_EventHandleGenerator"></a>

## Resource `EventHandleGenerator`



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



<pre><code><b>resource</b> <b>struct</b> <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T: <b>copyable</b>&gt;
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



<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_publish_generator">publish_generator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_publish_generator">publish_generator</a>(account: &signer) {
    move_to(account, <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>{ counter: 0, addr: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) })
}
</code></pre>



</details>

<a name="0x1_Event_fresh_guid"></a>

## Function `fresh_guid`



<pre><code><b>fun</b> <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <b>let</b> sender_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&counter.addr);
    <b>let</b> count_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&counter.counter);
    counter.counter = counter.counter + 1;

    // <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> goes first just in case we want <b>to</b> extend address in the future.
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> count_bytes, sender_bytes);

    count_bytes
}
</code></pre>



</details>

<a name="0x1_Event_new_event_handle"></a>

## Function `new_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;
<b>acquires</b> <a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; {
        counter: 0,
        guid: <a href="Event.md#0x1_Event_fresh_guid">fresh_guid</a>(borrow_global_mut&lt;<a href="Event.md#0x1_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)))
    }
}
</code></pre>



</details>

<a name="0x1_Event_emit_event"></a>

## Function `emit_event`



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



<pre><code><b>fun</b> <a href="Event.md#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Event.md#0x1_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T);
</code></pre>



</details>

<a name="0x1_Event_destroy_handle"></a>

## Function `destroy_handle`



<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Event.md#0x1_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt;) {
    <a href="Event.md#0x1_Event_EventHandle">EventHandle</a>&lt;T&gt; { counter: _, guid: _ } = handle;
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



Functions of the event module are mocked out using the intrinsic
pragma. They are implemented in the prover's prelude as no-ops.

Functionality in this module uses GUIDs created from serialization of
addresses and integers. These constructs are difficult to treat by the
verifier and the verification problem propagates up to callers of
those functions. Since events cannot be observed by Move programs,
mocking out functions of this module does not have effect on other
verification result.

A specification of the functions is neverthelesse  included in the
comments of this module and it has been verified.

> TODO(wrwg): We may want to have support by the Move prover to
> mock out functions for callers but still have them verified
> standlone.


<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>
