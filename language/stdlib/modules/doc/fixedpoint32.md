
<a name="0x0_FixedPoint32"></a>

# Module `0x0::FixedPoint32`

### Table of Contents

-  [Struct `T`](#0x0_FixedPoint32_T)
-  [Function `multiply_u64`](#0x0_FixedPoint32_multiply_u64)
-  [Function `divide_u64`](#0x0_FixedPoint32_divide_u64)
-  [Function `create_from_rational`](#0x0_FixedPoint32_create_from_rational)
-  [Function `create_from_raw_value`](#0x0_FixedPoint32_create_from_raw_value)
-  [Function `get_raw_value`](#0x0_FixedPoint32_get_raw_value)



<a name="0x0_FixedPoint32_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_FixedPoint32_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="#0x0_FixedPoint32_T">FixedPoint32::T</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="#0x0_FixedPoint32_T">T</a>): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication with u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (num <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // Convert back <b>to</b> u64. If the multiplier is larger than 1.0,
    // the value may be too large, which will cause the cast <b>to</b> fail
    // with an arithmetic error.
    (product <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x0_FixedPoint32_divide_u64"></a>

## Function `divide_u64`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="#0x0_FixedPoint32_T">FixedPoint32::T</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="#0x0_FixedPoint32_T">T</a>): u64 {
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (num <b>as</b> u128) &lt;&lt; 32;
    // Divide and convert the quotient <b>to</b> 64 bits. If the divisor is zero,
    // this will fail with a divide-by-zero error.
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // Convert back <b>to</b> u64. If the divisor is less than 1.0,
    // the value may be too large, which will cause the cast <b>to</b> fail
    // with an arithmetic error.
    (quotient <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x0_FixedPoint32_create_from_rational"></a>

## Function `create_from_rational`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x0_FixedPoint32_T">FixedPoint32::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x0_FixedPoint32_T">T</a> {
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    // If the denominator is zero, this will fail with a divide-by-zero
    // error.
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    // Check for underflow. Truncating <b>to</b> zero might be the desired result,
    // but <b>if</b> you really want a ratio of zero, it is easy <b>to</b> create that
    // from a raw value.
    Transaction::assert(quotient != 0 || numerator == 0, 16);
    // Return the quotient <b>as</b> a fixed-point number. The cast will fail
    // with an arithmetic error <b>if</b> the number is too large.
    <a href="#0x0_FixedPoint32_T">T</a> { value: (quotient <b>as</b> u64) }
}
</code></pre>



</details>

<a name="0x0_FixedPoint32_create_from_raw_value"></a>

## Function `create_from_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="#0x0_FixedPoint32_T">FixedPoint32::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="#0x0_FixedPoint32_T">T</a> {
    <a href="#0x0_FixedPoint32_T">T</a> { value }
}
</code></pre>



</details>

<a name="0x0_FixedPoint32_get_raw_value"></a>

## Function `get_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="#0x0_FixedPoint32_T">FixedPoint32::T</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="#0x0_FixedPoint32_T">T</a>): u64 {
    num.value
}
</code></pre>



</details>
