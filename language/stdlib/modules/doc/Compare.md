
<a name="0x1_Compare"></a>

# Module `0x1::Compare`

### Table of Contents

-  [Const `EQUAL`](#0x1_Compare_EQUAL)
-  [Const `LESS_THAN`](#0x1_Compare_LESS_THAN)
-  [Const `GREATER_THAN`](#0x1_Compare_GREATER_THAN)
-  [Function `cmp_lcs_bytes`](#0x1_Compare_cmp_lcs_bytes)
-  [Function `cmp_u8`](#0x1_Compare_cmp_u8)
-  [Function `cmp_u64`](#0x1_Compare_cmp_u64)

Utilities for comparing Move values based on their representation in LCS.


<a name="0x1_Compare_EQUAL"></a>

## Const `EQUAL`



<pre><code><b>const</b> EQUAL: u8 = 0;
</code></pre>



<a name="0x1_Compare_LESS_THAN"></a>

## Const `LESS_THAN`



<pre><code><b>const</b> LESS_THAN: u8 = 1;
</code></pre>



<a name="0x1_Compare_GREATER_THAN"></a>

## Const `GREATER_THAN`



<pre><code><b>const</b> GREATER_THAN: u8 = 2;
</code></pre>



<a name="0x1_Compare_cmp_lcs_bytes"></a>

## Function `cmp_lcs_bytes`

Compare vectors
<code>v1</code> and
<code>v2</code> using (1) vector contents from right to left and then
(2) vector length to break ties.
Returns either
<code>EQUAL</code> (0u8),
<code>LESS_THAN</code> (1u8), or
<code>GREATER_THAN</code> (2u8).

This function is designed to compare LCS (Libra Canonical Serialization)-encoded values
(i.e., vectors produced by
<code><a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a></code>). A typical client will call
<code><a href="#0x1_Compare_cmp_lcs_bytes">Compare::cmp_lcs_bytes</a>(<a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&t1), <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&t2)). The comparison provides the
following guarantees w.r.t the original values t1 and t2:
- </code>cmp_lcs_bytes(lcs(t1), lcs(t2)) == LESS_THAN
<code> iff </code>cmp_lcs_bytes(t2, t1) == GREATER_THAN
<code>
- </code>Compare::cmp<T>(t1, t2) == EQUAL
<code> iff </code>t1 == t2
<code> and (similarly)
</code>Compare::cmp<T>(t1, t2) != EQUAL
<code> iff </code>t1 != t2
<code>, where </code>==
<code> and </code>!=
<code> denote the Move
bytecode operations for polymorphic equality.
- for all primitive types </code>T
<code> with </code><
<code> and </code>>
<code> comparison operators exposed in Move bytecode
(</code>u8
<code>, </code>u64
<code>, </code>u128
<code>), we have
</code>compare_lcs_bytes(lcs(t1), lcs(t2)) == LESS_THAN
<code> iff </code>t1 < t2
<code> and (similarly)
</code>compare_lcs_bytes(lcs(t1), lcs(t2)) == LESS_THAN
<code> iff </code>t1 > t2
<code>.

For all other types, the order is whatever the <a href="LCS.md#0x1_LCS">LCS</a> encoding of the type and the comparison
strategy above gives you. One case where the order might be surprising is the </code>address
<code>
type.
<a href="CoreAddresses.md#0x1_CoreAddresses">CoreAddresses</a> are 16 byte hex values that <a href="LCS.md#0x1_LCS">LCS</a> encodes with the identity function. The right
<b>to</b> left, byte-by-byte comparison means that (for example)
</code>compare_lcs_bytes(lcs(0x01), lcs(0x10)) == LESS_THAN
<code> (<b>as</b> you'd expect), but
</code>compare_lcs_bytes(lcs(0x100), lcs(0x001)) == LESS_THAN` (as you probably wouldn't expect).
Keep this in mind when using this function to compare addresses.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Compare_cmp_lcs_bytes">cmp_lcs_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Compare_cmp_lcs_bytes">cmp_lcs_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8 {
    <b>let</b> i1 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v1);
    <b>let</b> i2 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v2);
    <b>let</b> len_cmp = <a href="#0x1_Compare_cmp_u64">cmp_u64</a>(i1, i2);

    // <a href="LCS.md#0x1_LCS">LCS</a> uses little endian encoding for all integer types, so we choose <b>to</b> compare from left
    // <b>to</b> right. Going right <b>to</b> left would make the behavior of <a href="#0x1_Compare">Compare</a>.cmp diverge from the
    // bytecode operators &lt; and &gt; on integer values (which would be confusing).
    <b>while</b> (i1 &gt; 0 && i2 &gt; 0) {
        i1 = i1 - 1;
        i2 = i2 - 1;
        <b>let</b> elem_cmp = <a href="#0x1_Compare_cmp_u8">cmp_u8</a>(*<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v1, i1), *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v2, i2));
        <b>if</b> (elem_cmp != 0) <b>return</b> elem_cmp
        // <b>else</b>, compare next element
    };
    // all compared elements equal; <b>use</b> length comparion <b>to</b> <b>break</b> the tie
    len_cmp
}
</code></pre>



</details>

<a name="0x1_Compare_cmp_u8"></a>

## Function `cmp_u8`

Compare two
<code>u8</code>'s


<pre><code><b>fun</b> <a href="#0x1_Compare_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Compare_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8 {
    <b>if</b> (i1 == i2) EQUAL
    <b>else</b> <b>if</b> (i1 &lt; i2) LESS_THAN
    <b>else</b> GREATER_THAN
}
</code></pre>



</details>

<a name="0x1_Compare_cmp_u64"></a>

## Function `cmp_u64`

Compare two
<code>u64</code>'s


<pre><code><b>fun</b> <a href="#0x1_Compare_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Compare_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8 {
    <b>if</b> (i1 == i2) EQUAL
    <b>else</b> <b>if</b> (i1 &lt; i2) LESS_THAN
    <b>else</b> GREATER_THAN
}
</code></pre>



</details>
