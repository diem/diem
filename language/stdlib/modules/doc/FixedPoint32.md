
<a name="0x1_FixedPoint32"></a>

# Module `0x1::FixedPoint32`

### Table of Contents

-  [Struct `FixedPoint32`](#0x1_FixedPoint32_FixedPoint32)
-  [Const `MAX_U64`](#0x1_FixedPoint32_MAX_U64)
-  [Const `EDENOMINATOR`](#0x1_FixedPoint32_EDENOMINATOR)
-  [Const `EDIVISION`](#0x1_FixedPoint32_EDIVISION)
-  [Const `EMULTIPLICATION`](#0x1_FixedPoint32_EMULTIPLICATION)
-  [Const `EDIVISION_BY_ZERO`](#0x1_FixedPoint32_EDIVISION_BY_ZERO)
-  [Const `ERATIO_OUT_OF_RANGE`](#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE)
-  [Function `multiply_u64`](#0x1_FixedPoint32_multiply_u64)
-  [Function `divide_u64`](#0x1_FixedPoint32_divide_u64)
-  [Function `create_from_rational`](#0x1_FixedPoint32_create_from_rational)
-  [Function `create_from_raw_value`](#0x1_FixedPoint32_create_from_raw_value)
-  [Function `get_raw_value`](#0x1_FixedPoint32_get_raw_value)
-  [Function `is_zero`](#0x1_FixedPoint32_is_zero)
-  [Specification](#0x1_FixedPoint32_Specification)
    -  [Function `multiply_u64`](#0x1_FixedPoint32_Specification_multiply_u64)
    -  [Function `divide_u64`](#0x1_FixedPoint32_Specification_divide_u64)
    -  [Function `create_from_rational`](#0x1_FixedPoint32_Specification_create_from_rational)



<a name="0x1_FixedPoint32_FixedPoint32"></a>

## Struct `FixedPoint32`

Define a fixed-point numeric type with 32 fractional bits.
This is just a u64 integer but it is wrapped in a struct to
make a unique type. This is a binary representation, so decimal
values may not be exactly representable, but it provides more
than 9 decimal digits of precision both before and after the
decimal point (18 digits total). For comparison, double precision
floating-point has less than 16 decimal digits of precision, so
be careful about using floating-point to convert these values to
decimal.


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

<a name="0x1_FixedPoint32_MAX_U64"></a>

## Const `MAX_U64`

TODO(wrwg): This should be provided somewhere centrally in the framework.


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_FixedPoint32_EDENOMINATOR"></a>

## Const `EDENOMINATOR`

The denominator provided was zero


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>: u64 = 0;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION"></a>

## Const `EDIVISION`

The quotient value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_EDIVISION">EDIVISION</a>: u64 = 1;
</code></pre>



<a name="0x1_FixedPoint32_EMULTIPLICATION"></a>

## Const `EMULTIPLICATION`

The multiplied value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>: u64 = 2;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION_BY_ZERO"></a>

## Const `EDIVISION_BY_ZERO`

A division by zero was encountered


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>: u64 = 3;
</code></pre>



<a name="0x1_FixedPoint32_ERATIO_OUT_OF_RANGE"></a>

## Const `ERATIO_OUT_OF_RANGE`

The computed ratio when converting to a <code><a href="#0x1_FixedPoint32">FixedPoint32</a></code> would be unrepresentable


<pre><code><b>const</b> <a href="#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>: u64 = 4;
</code></pre>



<a name="0x1_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`

Multiply a u64 integer by a fixed-point number, truncating any
fractional part of the product. This will abort if the product
overflows.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication with u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (val <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // Check whether the value is too large.
    <b>assert</b>(product &lt;= <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>));
    (product <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_divide_u64"></a>

## Function `divide_u64`

Divide a u64 integer by a fixed-point number, truncating any
fractional part of the quotient. This will abort if the divisor
is zero or if the quotient overflows.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // Check for division by zero.
    <b>assert</b>(divisor.value != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>));
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (val <b>as</b> u128) &lt;&lt; 32;
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // Check whether the value is too large.
    <b>assert</b>(quotient &lt;= <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="#0x1_FixedPoint32_EDIVISION">EDIVISION</a>));
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
raw value. This will abort if the denominator is zero. It will also
abort if the numerator is nonzero and the ratio is not in the range
2^-32 .. 2^32-1. When specifying decimal fractions, be careful about
rounding errors: if you round to display N digits after the decimal
point, you can use a denominator of 10^N to avoid numbers where the
very small imprecision in the binary representation could change the
rounding, e.g., 0.0125 will round down to 0.012 instead of up to 0.013.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32">FixedPoint32</a> {
    // If the denominator is zero, this will <b>abort</b>.
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    <b>assert</b>(scaled_denominator != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>));
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    <b>assert</b>(quotient != 0 || numerator == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
    // Return the quotient <b>as</b> a fixed-point number. We first need <b>to</b> check whether the cast
    // can succeed.
    <b>assert</b>(quotient &lt;= <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
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

<a name="0x1_FixedPoint32_is_zero"></a>

## Function `is_zero`

Returns true if the ratio is zero.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_is_zero">is_zero</a>(num: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_is_zero">is_zero</a>(num: <a href="#0x1_FixedPoint32">FixedPoint32</a>): bool {
    num.value == 0
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_Specification"></a>

## Specification


<a name="0x1_FixedPoint32_Specification_multiply_u64"></a>

### Function `multiply_u64`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_FixedPoint32_MultiplyAbortsIf">MultiplyAbortsIf</a>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val, multiplier);
</code></pre>




<a name="0x1_FixedPoint32_MultiplyAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_FixedPoint32_MultiplyAbortsIf">MultiplyAbortsIf</a> {
    val: num;
    multiplier: <a href="#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> <a href="#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val, multiplier) &gt; <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a> with <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_multiply_u64"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val: num, multiplier: <a href="#0x1_FixedPoint32">FixedPoint32</a>): num {
(val * multiplier.value) &gt;&gt; 32
}
</code></pre>



<a name="0x1_FixedPoint32_Specification_divide_u64"></a>

### Function `divide_u64`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_FixedPoint32_DividedAbortsIf">DividedAbortsIf</a>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val, divisor);
</code></pre>




<a name="0x1_FixedPoint32_DividedAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_FixedPoint32_DividedAbortsIf">DividedAbortsIf</a> {
    val: num;
    divisor: <a href="#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> divisor.value == 0 with <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val, divisor) &gt; <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a> with <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_divide_u64"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val: num, divisor: <a href="#0x1_FixedPoint32">FixedPoint32</a>): num {
(val &lt;&lt; 32) / divisor.value
}
</code></pre>



<a name="0x1_FixedPoint32_Specification_create_from_rational"></a>

### Function `create_from_rational`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_FixedPoint32_CreateFromRationalAbortsIf">CreateFromRationalAbortsIf</a>;
<b>ensures</b> result == <a href="#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator, denominator);
</code></pre>




<a name="0x1_FixedPoint32_CreateFromRationalAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_FixedPoint32_CreateFromRationalAbortsIf">CreateFromRationalAbortsIf</a> {
    numerator: u64;
    denominator: u64;
    <a name="0x1_FixedPoint32_scaled_numerator$9"></a>
    <b>let</b> scaled_numerator = numerator &lt;&lt; 64;
    <a name="0x1_FixedPoint32_scaled_denominator$10"></a>
    <b>let</b> scaled_denominator = denominator &lt;&lt; 32;
    <b>aborts_if</b> scaled_denominator == 0 with <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <a name="0x1_FixedPoint32_quotient$11"></a>
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    <b>aborts_if</b> (scaled_numerator / scaled_denominator) == 0 && scaled_numerator != 0 with <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> quotient &gt; <a href="#0x1_FixedPoint32_MAX_U64">MAX_U64</a> with <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_create_from_rational"></a>


<pre><code><b>define</b> <a href="#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator: num, denominator: num): <a href="#0x1_FixedPoint32">FixedPoint32</a> {
<a href="#0x1_FixedPoint32">FixedPoint32</a>{value: (numerator &lt;&lt; 64) / (denominator &lt;&lt; 32)}
}
</code></pre>




<pre><code>pragma verify;
pragma aborts_if_is_strict = <b>true</b>;
</code></pre>
