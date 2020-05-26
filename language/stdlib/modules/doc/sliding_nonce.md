
<a name="0x0_SlidingNonce"></a>

# Module `0x0::SlidingNonce`

### Table of Contents

-  [Struct `T`](#0x0_SlidingNonce_T)
-  [Function `record_nonce_or_abort`](#0x0_SlidingNonce_record_nonce_or_abort)
-  [Function `try_record_nonce`](#0x0_SlidingNonce_try_record_nonce)
-  [Function `publish_nonce_resource_for_user`](#0x0_SlidingNonce_publish_nonce_resource_for_user)



<a name="0x0_SlidingNonce_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_SlidingNonce_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>min_nonce: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>nonce_mask: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_SlidingNonce_record_nonce_or_abort"></a>

## Function `record_nonce_or_abort`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(seq_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(seq_nonce: u64) <b>acquires</b> <a href="#0x0_SlidingNonce_T">T</a> {
    <b>let</b> code = <a href="#0x0_SlidingNonce_try_record_nonce">try_record_nonce</a>(seq_nonce);
    Transaction::assert(code == 0, code);
}
</code></pre>



</details>

<a name="0x0_SlidingNonce_try_record_nonce"></a>

## Function `try_record_nonce`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_try_record_nonce">try_record_nonce</a>(seq_nonce: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_try_record_nonce">try_record_nonce</a>(seq_nonce: u64): u64 <b>acquires</b> <a href="#0x0_SlidingNonce_T">T</a> {
    <b>if</b> (seq_nonce == 0) {
        <b>return</b> 0
    };
    <b>let</b> t = borrow_global_mut&lt;<a href="#0x0_SlidingNonce_T">T</a>&gt;(Transaction::sender());
    <b>if</b> (t.min_nonce &gt; seq_nonce) {
        <b>return</b> 10001
    };
    <b>let</b> jump_limit = 10000; // Don't allow giant leaps in nonce <b>to</b> protect against nonce exhaustion
    <b>if</b> (t.min_nonce + jump_limit &lt;= seq_nonce) {
        <b>return</b> 10002
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>let</b> nonce_mask_size = 128; // size of T::nonce_mask in bits. no constants in <b>move</b>?
    <b>if</b> (bit_pos &gt;= nonce_mask_size) {
        <b>let</b> shift = (bit_pos - nonce_mask_size + 1);
        <b>if</b>(shift &gt;= nonce_mask_size) {
            t.nonce_mask = 0;
            t.min_nonce = seq_nonce + 1 - nonce_mask_size;
        } <b>else</b> {
            t.nonce_mask = t.nonce_mask &gt;&gt; (shift <b>as</b> u8);
            t.min_nonce = t.min_nonce + shift;
        }
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>let</b> set = 1u128 &lt;&lt; (bit_pos <b>as</b> u8);
    <b>if</b> (t.nonce_mask & set != 0) {
        <b>return</b> 10003
    };
    t.nonce_mask = t.nonce_mask | set;
    0
}
</code></pre>



</details>

<a name="0x0_SlidingNonce_publish_nonce_resource_for_user"></a>

## Function `publish_nonce_resource_for_user`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_publish_nonce_resource_for_user">publish_nonce_resource_for_user</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SlidingNonce_publish_nonce_resource_for_user">publish_nonce_resource_for_user</a>() {
    <b>let</b> new_resource = <a href="#0x0_SlidingNonce_T">T</a> {
        min_nonce: 0,
        nonce_mask: 0,
    };
    move_to_sender(new_resource)
}
</code></pre>



</details>
