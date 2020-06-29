
<a name="0x1_FixedPoint32"></a>

# Module `0x1::FixedPoint32`

### Table of Contents

-  [Struct `FixedPoint32`](#0x1_FixedPoint32_FixedPoint32)
-  [Function `multiply_u64`](#0x1_FixedPoint32_multiply_u64)
-  [Function `divide_u64`](#0x1_FixedPoint32_divide_u64)
-  [Function `create_from_rational`](#0x1_FixedPoint32_create_from_rational)
-  [Function `create_from_raw_value`](#0x1_FixedPoint32_create_from_raw_value)
-  [Function `get_raw_value`](#0x1_FixedPoint32_get_raw_value)
-  [Specification](#0x1_FixedPoint32_Specification)
    -  [Function `multiply_u64`](#0x1_FixedPoint32_Specification_multiply_u64)
    -  [Function `divide_u64`](#0x1_FixedPoint32_Specification_divide_u64)
    -  [Function `create_from_rational`](#0x1_FixedPoint32_Specification_create_from_rational)



<a name="0x1_FixedPoint32_FixedPoint32"></a>

## Struct `FixedPoint32`

Define a fixed-point numeric type with 32 fractional bits.
This is just a u64 integer but it is wrapped in a struct to
make a unique type.


<pre><code><b>struct</b> <a href="#0x1_FixedPoint32">FixedPoint32</a>
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

<a name="0x1_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`

Multiply a u64 integer by a fixed-point number, truncating any
fractional part of the product. This will abort if the product
overflows.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication with u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (num <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // the value may be too large, which will cause the cast <b>to</b> fail
    // with an arithmetic error.
    (product <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_divide_u64"></a>

## Function `divide_u64`

Divide a u64 integer by a fixed-point number, truncating any
fractional part of the quotient. This will abort if the divisor
is zero or if the quotient overflows.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (num <b>as</b> u128) &lt;&lt; 32;
    // this will fail with a divide-by-zero error.
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // the value may be too large, which will cause the cast <b>to</b> fail
    // with an arithmetic error.
    (quotient <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_rational"></a>

## Function `create_from_rational`

Create a fixed-point value from a rational number specified by its
numerator and denominator. This function is for convenience; it is also
perfectly fine to create a fixed-point value by directly specifying the
raw value. This will abort if the denominator is zero or if the ratio is
not in the range 2^-32 .. 2^32-1.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32">FixedPoint32</a> {
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    // If the denominator is zero, this will fail with a divide-by-zero
    // error.
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    // but <b>if</b> you really want a ratio of zero, it is easy <b>to</b> create that
    // from a raw value.
    <b>assert</b>(quotient != 0 || numerator == 0, 16);
    // Return the quotient <b>as</b> a fixed-point number. The cast will fail
    // with an arithmetic error <b>if</b> the number is too large.
    <a href="#0x1_FixedPoint32">FixedPoint32</a> { value: (quotient <b>as</b> u64) }
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_raw_value"></a>

## Function `create_from_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="#0x1_FixedPoint32">FixedPoint32</a> {
    <a href="#0x1_FixedPoint32">FixedPoint32</a> { value }
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_get_raw_value"></a>

## Function `get_raw_value`

Accessor for the raw u64 value. Other less common operations, such as
adding or subtracting FixedPoint32 values, can be done using the raw
values directly.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    num.value
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_Specification"></a>

## Specification


<a name="0x1_FixedPoint32_Specification_multiply_u64"></a>

### Function `multiply_u64`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



Currently, we ignore the actual implementation of this function in verification
and treat it as uninterpreted, which simplifies the verification problem significantly.
This way we avoid the non-linear arithmetic problem presented by this function.

Abstracting this and related functions is possible because the correctness of currency
conversion (where
<code><a href="#0x1_FixedPoint32">FixedPoint32</a></code> is used for) is not relevant for the rest of the contract
control flow, so we can assume some arbitrary (but fixed) behavior here.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(num, multiplier);
</code></pre>



<a name="0x1_FixedPoint32_Specification_divide_u64"></a>

### Function `divide_u64`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



See comment at
<code>Self::multiply_64</code>.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(num, divisor);
</code></pre>



<a name="0x1_FixedPoint32_Specification_create_from_rational"></a>

### Function `create_from_rational`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



See comment at
<code>Self::multiply_64</code>.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator, denominator);
</code></pre>



Uninterpreted function for
<code><a href="#0x1_FixedPoint32_multiply_u64">Self::multiply_u64</a></code>.


<a name="0x1_FixedPoint32_spec_multiply_u64"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val: u64, multiplier: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for
<code><a href="#0x1_FixedPoint32_divide_u64">Self::divide_u64</a></code>.


<a name="0x1_FixedPoint32_spec_divide_u64"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val: u64, divisor: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for
<code><a href="#0x1_FixedPoint32_create_from_rational">Self::create_from_rational</a></code>.


<a name="0x1_FixedPoint32_spec_create_from_rational"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32">FixedPoint32</a>;
</code></pre>
