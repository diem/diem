
<a name="0x0_Event"></a>

# Module `0x0::Event`

### Table of Contents

-  [Struct `EventHandleGenerator`](#0x0_Event_EventHandleGenerator)
-  [Struct `EventHandle`](#0x0_Event_EventHandle)
-  [Function `publish_generator`](#0x0_Event_publish_generator)
-  [Function `fresh_guid`](#0x0_Event_fresh_guid)
-  [Function `new_event_handle`](#0x0_Event_new_event_handle)
-  [Function `emit_event`](#0x0_Event_emit_event)
-  [Function `write_to_event_store`](#0x0_Event_write_to_event_store)
-  [Function `destroy_handle`](#0x0_Event_destroy_handle)



<a name="0x0_Event_EventHandleGenerator"></a>

## Struct `EventHandleGenerator`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>
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

<a name="0x0_Event_EventHandle"></a>

## Struct `EventHandle`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T: <b>copyable</b>&gt;
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

<a name="0x0_Event_publish_generator"></a>

## Function `publish_generator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_publish_generator">publish_generator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_publish_generator">publish_generator</a>(account: &signer) {
    move_to(account, <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>{ counter: 0, addr: <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) })
}
</code></pre>



</details>

<a name="0x0_Event_fresh_guid"></a>

## Function `fresh_guid`



<pre><code><b>fun</b> <a href="#0x0_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="#0x0_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Event_fresh_guid">fresh_guid</a>(counter: &<b>mut</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>): vector&lt;u8&gt; {
    <b>let</b> sender_bytes = <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(&counter.addr);
    <b>let</b> count_bytes = <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(&counter.counter);
    counter.counter = counter.counter + 1;

    // <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> goes first just in case we want <b>to</b> extend address in the future.
    <a href="vector.md#0x0_Vector_append">Vector::append</a>(&<b>mut</b> count_bytes, sender_bytes);

    count_bytes
}
</code></pre>



</details>

<a name="0x0_Event_new_event_handle"></a>

## Function `new_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="#0x0_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(account: &signer): <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt;
<b>acquires</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt; {
        counter: 0,
        guid: <a href="#0x0_Event_fresh_guid">fresh_guid</a>(borrow_global_mut&lt;<a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(account)))
    }
}
</code></pre>



</details>

<a name="0x0_Event_emit_event"></a>

## Function `emit_event`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="#0x0_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_emit_event">emit_event</a>&lt;T: <b>copyable</b>&gt;(handle_ref: &<b>mut</b> <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt;, msg: T) {
    <b>let</b> guid = *&handle_ref.guid;

    <a href="#0x0_Event_write_to_event_store">write_to_event_store</a>&lt;T&gt;(guid, handle_ref.counter, msg);
    handle_ref.counter = handle_ref.counter + 1;
}
</code></pre>



</details>

<a name="0x0_Event_write_to_event_store"></a>

## Function `write_to_event_store`



<pre><code><b>fun</b> <a href="#0x0_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x0_Event_write_to_event_store">write_to_event_store</a>&lt;T: <b>copyable</b>&gt;(guid: vector&lt;u8&gt;, count: u64, msg: T);
</code></pre>



</details>

<a name="0x0_Event_destroy_handle"></a>

## Function `destroy_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="#0x0_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_destroy_handle">destroy_handle</a>&lt;T: <b>copyable</b>&gt;(handle: <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt;) {
    <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt; { counter: _, guid: _ } = handle;
}
</code></pre>



</details>
