
<a name="0x1_BitVector"></a>

# Module `0x1::BitVector`



-  [Struct `BitVector`](#0x1_BitVector_BitVector)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x1_BitVector_new)
-  [Function `set`](#0x1_BitVector_set)
-  [Function `unset`](#0x1_BitVector_unset)
-  [Function `shift_left`](#0x1_BitVector_shift_left)
-  [Function `is_index_set`](#0x1_BitVector_is_index_set)
-  [Function `bit_index`](#0x1_BitVector_bit_index)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="">0x1::Vector</a>;
</code></pre>



<a name="0x1_BitVector_BitVector"></a>

## Struct `BitVector`



<pre><code><b>struct</b> <a href="BitVector.md#0x1_BitVector">BitVector</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>length: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>bit_field: vector&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_BitVector_EINDEX"></a>

The provided index is out of bounds


<pre><code><b>const</b> <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>: u64 = 0;
</code></pre>



<a name="0x1_BitVector_WORD_SIZE"></a>



<pre><code><b>const</b> <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>: u64 = 64;
</code></pre>



<a name="0x1_BitVector_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_new">new</a>(length: u64): <a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_new">new</a>(length: u64): <a href="BitVector.md#0x1_BitVector">BitVector</a> {
    <b>let</b> num_words = (length + (<a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a> - 1)) /  <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>;
    <b>let</b> bit_field = <a href="_empty">Vector::empty</a>();
    <b>while</b> (num_words &gt; 0) {
        <a href="_push_back">Vector::push_back</a>(&<b>mut</b> bit_field, 0u64);
        num_words = num_words - 1;
    };

    <a href="BitVector.md#0x1_BitVector">BitVector</a> {
        length,
        bit_field,
    }
}
</code></pre>



</details>

<a name="0x1_BitVector_set"></a>

## Function `set`

Set the bit at <code>bit_index</code> in the <code>bitvector</code> regardless of its previous state.


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_set">set</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>, bit_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_set">set</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector">BitVector</a>, bit_index: u64) {
    <b>assert</b>(<a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a> &lt; bitvector.length, <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>);
    <b>let</b> (inner_index, inner) = <a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a>(bitvector, bit_index);
    *inner = *inner | 1u64 &lt;&lt; (inner_index <b>as</b> u8);
}
</code></pre>



</details>

<a name="0x1_BitVector_unset"></a>

## Function `unset`

Unset the bit at <code>bit_index</code> in the <code>bitvector</code> regardless of its previous state.


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_unset">unset</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>, bit_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_unset">unset</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector">BitVector</a>, bit_index: u64) {
    <b>assert</b>(<a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a> &lt; bitvector.length, <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>);
    <b>let</b> (inner_index, inner) = <a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a>(bitvector, bit_index);
    // Having negation would be nice here...
    *inner = *inner ^ (*inner & (1u64 &lt;&lt; (inner_index <b>as</b> u8)));
}
</code></pre>



</details>

<a name="0x1_BitVector_shift_left"></a>

## Function `shift_left`

Shift the <code>bitvector</code> left by <code>amount</code>, <code>amount</code> must be less than the
bitvector's length.


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_shift_left">shift_left</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_shift_left">shift_left</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector">BitVector</a>, amount: u64) {
    <b>assert</b>(amount &lt; bitvector.length, <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>);
    <b>let</b> i = amount;

    <b>while</b> (i &lt; bitvector.length) {
        <b>if</b> (<a href="BitVector.md#0x1_BitVector_is_index_set">is_index_set</a>(bitvector, i)) <a href="BitVector.md#0x1_BitVector_set">set</a>(bitvector, i - amount)
        <b>else</b> <a href="BitVector.md#0x1_BitVector_unset">unset</a>(bitvector, i - amount);
        i = i + 1;
    };

    i = bitvector.length - amount;

    <b>while</b> (i &lt; bitvector.length) {
        <a href="BitVector.md#0x1_BitVector_unset">unset</a>(bitvector, i);
        i = i + 1;
    };
}
</code></pre>



</details>

<a name="0x1_BitVector_is_index_set"></a>

## Function `is_index_set`

Return the value of the bit at <code>bit_index</code> in the <code>bitvector</code>. <code><b>true</b></code>
represents "1" and <code><b>false</b></code> represents a 0


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_is_index_set">is_index_set</a>(bitvector: &<a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>, bit_index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BitVector.md#0x1_BitVector_is_index_set">is_index_set</a>(bitvector: &<a href="BitVector.md#0x1_BitVector">BitVector</a>, bit_index: u64): bool {
    <b>assert</b>(<a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a> &lt; bitvector.length, <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>);
    <b>let</b> inner = <a href="_borrow">Vector::borrow</a>(&bitvector.bit_field, bit_index / <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>);
    <b>let</b> inner_index = bit_index % <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>;
    *inner & (1 &lt;&lt; (inner_index <b>as</b> u8)) != 0
}
</code></pre>



</details>

<a name="0x1_BitVector_bit_index"></a>

## Function `bit_index`

Return the larger containing u64, and the bit index within that u64
for <code>index</code> w.r.t. <code>bitvector</code>.


<pre><code><b>fun</b> <a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a>, index: u64): (u64, &<b>mut</b> u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="BitVector.md#0x1_BitVector_bit_index">bit_index</a>(bitvector: &<b>mut</b> <a href="BitVector.md#0x1_BitVector">BitVector</a>, index: u64): (u64, &<b>mut</b> u64) {
    <b>assert</b>(index &lt; bitvector.length, <a href="BitVector.md#0x1_BitVector_EINDEX">EINDEX</a>);
    (index % <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>, <a href="_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> bitvector.bit_field, index / <a href="BitVector.md#0x1_BitVector_WORD_SIZE">WORD_SIZE</a>))
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
