
<a name="0x0_LibraTimestamp"></a>

# Module `0x0::LibraTimestamp`

### Table of Contents

-  [Struct `CurrentTimeMicroseconds`](#0x0_LibraTimestamp_CurrentTimeMicroseconds)
-  [Function `initialize`](#0x0_LibraTimestamp_initialize)
-  [Function `update_global_time`](#0x0_LibraTimestamp_update_global_time)
-  [Function `now_microseconds`](#0x0_LibraTimestamp_now_microseconds)
-  [Function `is_genesis`](#0x0_LibraTimestamp_is_genesis)



<a name="0x0_LibraTimestamp_CurrentTimeMicroseconds"></a>

## Struct `CurrentTimeMicroseconds`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>microseconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraTimestamp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_initialize">initialize</a>() {
    // Only callable by the <a href="association.md#0x0_Association">Association</a> address
    Transaction::assert(Transaction::sender() == 0xA550C18, 1);

    // TODO: Should the initialized value be passed in <b>to</b> genesis?
    <b>let</b> timer = <a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {microseconds: 0};
    move_to_sender&lt;<a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(timer);
}
</code></pre>



</details>

<a name="0x0_LibraTimestamp_update_global_time"></a>

## Function `update_global_time`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_update_global_time">update_global_time</a>(proposer: address, timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_update_global_time">update_global_time</a>(proposer: address, timestamp: u64) <b>acquires</b> <a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    // Can only be invoked by LibraVM privilege.
    Transaction::assert(Transaction::sender() == 0x0, 33);

    <b>let</b> global_timer = borrow_global_mut&lt;<a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(0xA550C18);
    <b>if</b> (proposer == 0x0) {
        // NIL block with null address <b>as</b> proposer. Timestamp must be equal.
        Transaction::assert(timestamp == global_timer.microseconds, 5001);
    } <b>else</b> {
        // Normal block. Time must advance
        Transaction::assert(global_timer.microseconds &lt; timestamp, 5001);
    };
    global_timer.microseconds = timestamp;
}
</code></pre>



</details>

<a name="0x0_LibraTimestamp_now_microseconds"></a>

## Function `now_microseconds`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64 <b>acquires</b> <a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    borrow_global&lt;<a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(0xA550C18).microseconds
}
</code></pre>



</details>

<a name="0x0_LibraTimestamp_is_genesis"></a>

## Function `is_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTimestamp_is_genesis">is_genesis</a>(): bool {
    !::exists&lt;<a href="#0x0_LibraTimestamp_CurrentTimeMicroseconds">Self::CurrentTimeMicroseconds</a>&gt;(0xA550C18)
}
</code></pre>



</details>
