
<a name="0x1_BCS"></a>

# Module `0x1::BCS`

Utility for converting a Move value to its binary representation in BCS (Binary Canonical
Serialization). BCS is the binary encoding for Move resources and other non-module values
published on-chain. See https://github.com/diem/bcs#binary-canonical-serialization-bcs for more
details on BCS.


-  [Constants](#@Constants_0)
-  [Function `to_bytes`](#0x1_BCS_to_bytes)
-  [Function `cmp_bytes`](#0x1_BCS_cmp_bytes)
-  [Function `cmp_u8`](#0x1_BCS_cmp_u8)
-  [Function `cmp_u64`](#0x1_BCS_cmp_u64)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_BCS_EQUAL"></a>



<pre><code><b>const</b> <a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a>: u8 = 0;
</code></pre>



<a name="0x1_BCS_GREATER_THAN"></a>



<pre><code><b>const</b> <a href="BCS.md#0x1_BCS_GREATER_THAN">GREATER_THAN</a>: u8 = 2;
</code></pre>



<a name="0x1_BCS_LESS_THAN"></a>



<pre><code><b>const</b> <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a>: u8 = 1;
</code></pre>



<a name="0x1_BCS_to_bytes"></a>

## Function `to_bytes`

Return the binary representation of <code>v</code> in BCS (Binary Canonical Serialization) format


<pre><code><b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_BCS_cmp_bytes"></a>

## Function `cmp_bytes`

Compare vectors <code>v1</code> and <code>v2</code> using (1) vector contents from right to left and then
(2) vector length to break ties.
Returns either <code><a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a></code> (0u8), <code><a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> (1u8), or <code><a href="BCS.md#0x1_BCS_GREATER_THAN">GREATER_THAN</a></code> (2u8).

This function is designed to compare BCS (Binary Canonical Serialization)-encoded values
(i.e., vectors produced by <code><a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a></code>). A typical client will call
<code>Compare::cmp_bcs_bytes(<a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(&t1), <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(&t2))</code>. The comparison provides the
following guarantees w.r.t the original values t1 and t2:
- <code>cmp_bcs_bytes(bcs(t1), bcs(t2)) == <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> iff <code>cmp_bcs_bytes(t2, t1) == <a href="BCS.md#0x1_BCS_GREATER_THAN">GREATER_THAN</a></code>
- <code>Compare::cmp&lt;T&gt;(t1, t2) == <a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a></code> iff <code>t1 == t2</code> and (similarly)
<code>Compare::cmp&lt;T&gt;(t1, t2) != <a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a></code> iff <code>t1 != t2</code>, where <code>==</code> and <code>!=</code> denote the Move
bytecode operations for polymorphic equality.
- for all primitive types <code>T</code> with <code>&lt;</code> and <code>&gt;</code> comparison operators exposed in Move bytecode
(<code>u8</code>, <code>u64</code>, <code>u128</code>), we have
<code>compare_bcs_bytes(bcs(t1), bcs(t2)) == <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> iff <code>t1 &lt; t2</code> and (similarly)
<code>compare_bcs_bytes(bcs(t1), bcs(t2)) == <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> iff <code>t1 &gt; t2</code>.

For all other types, the order is whatever the BCS encoding of the type and the comparison
strategy above gives you. One case where the order might be surprising is the <code>address</code>
type.
Move <code>address</code>es are 16 byte hex values that BCS encodes with the identity function. The right
to left, byte-by-byte comparison means that (for example)
<code>compare_bcs_bytes(bcs(0x01), bcs(0x10)) == <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> (as you'd expect), but
<code>compare_bcs_bytes(bcs(0x100), bcs(0x001)) == <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a></code> (as you probably wouldn't expect).
Keep this in mind when using this function to compare addresses.

> TODO: there is currently no specification for this function, which causes no problem because it is not yet
> used in the Diem framework. However, should this functionality be needed in specification, a customized
> native abstraction is needed in the prover framework.


<pre><code><b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_cmp_bytes">cmp_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_cmp_bytes">cmp_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8 {
    <b>let</b> i1 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v1);
    <b>let</b> i2 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v2);
    <b>let</b> len_cmp = <a href="BCS.md#0x1_BCS_cmp_u64">cmp_u64</a>(i1, i2);

    // <a href="BCS.md#0x1_BCS">BCS</a> uses little endian encoding for all integer types, so we choose <b>to</b> compare from left
    // <b>to</b> right. Going right <b>to</b> left would make the behavior of Compare.cmp diverge from the
    // bytecode operators &lt; and &gt; on integer values (which would be confusing).
    <b>while</b> (i1 &gt; 0 && i2 &gt; 0) {
        i1 = i1 - 1;
        i2 = i2 - 1;
        <b>let</b> elem_cmp = <a href="BCS.md#0x1_BCS_cmp_u8">cmp_u8</a>(*<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v1, i1), *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v2, i2));
        <b>if</b> (elem_cmp != 0) <b>return</b> elem_cmp
        // <b>else</b>, compare next element
    };
    // all compared elements equal; <b>use</b> length comparison <b>to</b> <b>break</b> the tie
    len_cmp
}
</code></pre>



</details>

<a name="0x1_BCS_cmp_u8"></a>

## Function `cmp_u8`

Compare two <code>u8</code>'s


<pre><code><b>fun</b> <a href="BCS.md#0x1_BCS_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="BCS.md#0x1_BCS_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8 {
    <b>if</b> (i1 == i2) <a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a>
    <b>else</b> <b>if</b> (i1 &lt; i2) <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a>
    <b>else</b> <a href="BCS.md#0x1_BCS_GREATER_THAN">GREATER_THAN</a>
}
</code></pre>



</details>

<a name="0x1_BCS_cmp_u64"></a>

## Function `cmp_u64`

Compare two <code>u64</code>'s


<pre><code><b>fun</b> <a href="BCS.md#0x1_BCS_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="BCS.md#0x1_BCS_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8 {
    <b>if</b> (i1 == i2) <a href="BCS.md#0x1_BCS_EQUAL">EQUAL</a>
    <b>else</b> <b>if</b> (i1 &lt; i2) <a href="BCS.md#0x1_BCS_LESS_THAN">LESS_THAN</a>
    <b>else</b> <a href="BCS.md#0x1_BCS_GREATER_THAN">GREATER_THAN</a>
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



Native function which is defined in the prover's prelude.


<a name="0x1_BCS_serialize"></a>


<pre><code><b>native</b> <b>fun</b> <a href="BCS.md#0x1_BCS_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
