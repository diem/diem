
<a name="0x0_Event"></a>

# Module `0x0::Event`

### Table of Contents

-  [Struct `EventHandleGeneratorCreationCapability`](#0x0_Event_EventHandleGeneratorCreationCapability)
-  [Struct `EventHandleGenerator`](#0x0_Event_EventHandleGenerator)
-  [Struct `EventHandle`](#0x0_Event_EventHandle)
-  [Function `grant_event_handle_creation_operation`](#0x0_Event_grant_event_handle_creation_operation)
-  [Function `grant_event_generator`](#0x0_Event_grant_event_generator)
-  [Function `new_event_generator`](#0x0_Event_new_event_generator)
-  [Function `fresh_guid`](#0x0_Event_fresh_guid)
-  [Function `new_event_handle_from_generator`](#0x0_Event_new_event_handle_from_generator)
-  [Function `new_event_handle`](#0x0_Event_new_event_handle)
-  [Function `emit_event`](#0x0_Event_emit_event)
-  [Function `write_to_event_store`](#0x0_Event_write_to_event_store)
-  [Function `destroy_handle`](#0x0_Event_destroy_handle)



<a name="0x0_Event_EventHandleGeneratorCreationCapability"></a>

## Struct `EventHandleGeneratorCreationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Event_EventHandleGeneratorCreationCapability">EventHandleGeneratorCreationCapability</a>
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

<a name="0x0_Event_grant_event_handle_creation_operation"></a>

## Function `grant_event_handle_creation_operation`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_grant_event_handle_creation_operation">grant_event_handle_creation_operation</a>(): <a href="#0x0_Event_EventHandleGeneratorCreationCapability">Event::EventHandleGeneratorCreationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_grant_event_handle_creation_operation">grant_event_handle_creation_operation</a>(): <a href="#0x0_Event_EventHandleGeneratorCreationCapability">EventHandleGeneratorCreationCapability</a> {
    Transaction::assert(Transaction::sender() == 0xA550C18, 0);
    <a href="#0x0_Event_EventHandleGeneratorCreationCapability">EventHandleGeneratorCreationCapability</a>{}
}
</code></pre>



</details>

<a name="0x0_Event_grant_event_generator"></a>

## Function `grant_event_generator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_grant_event_generator">grant_event_generator</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_grant_event_generator">grant_event_generator</a>() {
    Transaction::assert(<a href="libra_time.md#0x0_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), 0);
    move_to_sender(<a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> { counter: 0, addr: Transaction::sender() })
}
</code></pre>



</details>

<a name="0x0_Event_new_event_generator"></a>

## Function `new_event_generator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_generator">new_event_generator</a>(addr: address, _cap: &<a href="#0x0_Event_EventHandleGeneratorCreationCapability">Event::EventHandleGeneratorCreationCapability</a>): <a href="#0x0_Event_EventHandleGenerator">Event::EventHandleGenerator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_generator">new_event_generator</a>(
    addr: address,
    _cap: &<a href="#0x0_Event_EventHandleGeneratorCreationCapability">EventHandleGeneratorCreationCapability</a>
): <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> <b>acquires</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <b>if</b> (::exists&lt;<a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr) && <a href="libra_time.md#0x0_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>()) {
        // <b>if</b> the account already has an event handle generator, <b>return</b> it instead of creating
        // a new one. the reason: it may have already been used <b>to</b> generate event handles and
        // thus may have a nonzero `counter`.
        // this should only happen during genesis bootstrapping, and only for the association
        // account and the config account.
        // TODO: see <b>if</b> we can eliminate this hack + the initialize() function
        Transaction::assert(Transaction::sender() == 0xA550C18 || Transaction::sender() == 0xF1A95, 0);
        Transaction::assert(addr == 0xA550C18 || addr == 0xF1A95, 0);

        move_from&lt;<a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(addr)
    } <b>else</b> {
        <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>{ counter: 0, addr }
    }
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

<a name="0x0_Event_new_event_handle_from_generator"></a>

## Function `new_event_handle_from_generator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle_from_generator">new_event_handle_from_generator</a>&lt;T: <b>copyable</b>&gt;(event_generator: &<b>mut</b> <a href="#0x0_Event_EventHandleGenerator">Event::EventHandleGenerator</a>): <a href="#0x0_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle_from_generator">new_event_handle_from_generator</a>&lt;T: <b>copyable</b>&gt;(event_generator: &<b>mut</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>): <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt; {
    <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt; {counter: 0, guid: <a href="#0x0_Event_fresh_guid">fresh_guid</a>(event_generator)}
}
</code></pre>



</details>

<a name="0x0_Event_new_event_handle"></a>

## Function `new_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(): <a href="#0x0_Event_EventHandle">Event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Event_new_event_handle">new_event_handle</a>&lt;T: <b>copyable</b>&gt;(): <a href="#0x0_Event_EventHandle">EventHandle</a>&lt;T&gt;
<b>acquires</b> <a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a> {
    <b>let</b> event_generator = borrow_global_mut&lt;<a href="#0x0_Event_EventHandleGenerator">EventHandleGenerator</a>&gt;(Transaction::sender());
    <a href="#0x0_Event_new_event_handle_from_generator">new_event_handle_from_generator</a>(event_generator)
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
