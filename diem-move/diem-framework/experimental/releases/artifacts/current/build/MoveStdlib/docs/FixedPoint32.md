
<a name="0x1_FixedPoint32"></a>

# Module `0x1::FixedPoint32`

Defines a fixed-point numeric type with a 32-bit integer part and
a 32-bit fractional part.


-  [Struct `FixedPoint32`](#0x1_FixedPoint32_FixedPoint32)
-  [Constants](#@Constants_0)
-  [Function `multiply_u64`](#0x1_FixedPoint32_multiply_u64)
    -  [Abstract Semantics](#@Abstract_Semantics_1)
-  [Function `divide_u64`](#0x1_FixedPoint32_divide_u64)
    -  [Abstract Semantics](#@Abstract_Semantics_2)
-  [Function `create_from_rational`](#0x1_FixedPoint32_create_from_rational)
    -  [Abstract Semantics](#@Abstract_Semantics_3)
-  [Function `create_from_raw_value`](#0x1_FixedPoint32_create_from_raw_value)
-  [Function `get_raw_value`](#0x1_FixedPoint32_get_raw_value)
-  [Function `is_zero`](#0x1_FixedPoint32_is_zero)
-  [Module Specification](#@Module_Specification_4)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



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


<pre><code><b>struct</b> <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> has <b>copy</b>, drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_FixedPoint32_MAX_U64"></a>

> TODO: This is a basic constant and should be provided somewhere centrally in the framework.


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_FixedPoint32_EDENOMINATOR"></a>

The denominator provided was zero


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>: u64 = 0;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION"></a>

The quotient value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION">EDIVISION</a>: u64 = 1;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION_BY_ZERO"></a>

A division by zero was encountered


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>: u64 = 3;
</code></pre>



<a name="0x1_FixedPoint32_EMULTIPLICATION"></a>

The multiplied value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>: u64 = 2;
</code></pre>



<a name="0x1_FixedPoint32_ERATIO_OUT_OF_RANGE"></a>

The computed ratio when converting to a <code><a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code> would be unrepresentable


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>: u64 = 4;
</code></pre>



<a name="0x1_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`

Multiply a u64 integer by a fixed-point number, truncating any
fractional part of the product. This will abort if the product
overflows.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication <b>with</b> u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (val <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // Check whether the value is too large.
    <b>assert</b>!(product &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>));
    (product <b>as</b> u64)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>

Because none of our SMT solvers supports non-linear arithmetic with reliable efficiency,
we specify the concrete semantics of the implementation but use
an abstracted, simplified semantics for verification of callers. For the verification outcome of
callers, the actual result of this function is not relevant, as long as the abstraction behaves
homomorphic. This does not guarantee that arithmetic functions using this code is correct.


<pre><code><b>pragma</b> opaque;
<b>include</b> [concrete] <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteMultiplyAbortsIf">ConcreteMultiplyAbortsIf</a>;
<b>ensures</b> [concrete] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_multiply_u64">spec_concrete_multiply_u64</a>(val, multiplier);
<b>include</b> [abstract] <a href="FixedPoint32.md#0x1_FixedPoint32_MultiplyAbortsIf">MultiplyAbortsIf</a>;
<b>ensures</b> [abstract] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val, multiplier);
</code></pre>




<a name="0x1_FixedPoint32_ConcreteMultiplyAbortsIf"></a>


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteMultiplyAbortsIf">ConcreteMultiplyAbortsIf</a> {
    val: num;
    multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_multiply_u64">spec_concrete_multiply_u64</a>(val, multiplier) &gt; <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_concrete_multiply_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_multiply_u64">spec_concrete_multiply_u64</a>(val: num, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): num {
   (val * multiplier.value) &gt;&gt; 32
}
</code></pre>



<a name="@Abstract_Semantics_1"></a>

### Abstract Semantics



<a name="0x1_FixedPoint32_MultiplyAbortsIf"></a>


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_MultiplyAbortsIf">MultiplyAbortsIf</a> {
    val: num;
    multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val, multiplier) &gt; <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_multiply_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val: num, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): num {
   <b>if</b> (multiplier.value == 0)
       // Zero value
       0
   <b>else</b> <b>if</b> (multiplier.value == 1)
       // 1.0
       val
   <b>else</b> <b>if</b> (multiplier.value == 2)
       // 0.5
       val / 2
   <b>else</b>
       // overflow
       <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> + 1
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_divide_u64"></a>

## Function `divide_u64`

Divide a u64 integer by a fixed-point number, truncating any
fractional part of the quotient. This will abort if the divisor
is zero or if the quotient overflows.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // Check for division by zero.
    <b>assert</b>!(divisor.value != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>));
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (val <b>as</b> u128) &lt;&lt; 32;
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // Check whether the value is too large.
    <b>assert</b>!(quotient &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION">EDIVISION</a>));
    // the value may be too large, which will cause the cast <b>to</b> fail
    // <b>with</b> an arithmetic error.
    (quotient <b>as</b> u64)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>

We specify the concrete semantics of the implementation but use
an abstracted, simplified semantics for verification of callers.


<pre><code><b>pragma</b> opaque;
<b>include</b> [concrete] <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteDivideAbortsIf">ConcreteDivideAbortsIf</a>;
<b>ensures</b> [concrete] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_divide_u64">spec_concrete_divide_u64</a>(val, divisor);
<b>include</b> [abstract] <a href="FixedPoint32.md#0x1_FixedPoint32_DivideAbortsIf">DivideAbortsIf</a>;
<b>ensures</b> [abstract] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val, divisor);
</code></pre>




<a name="0x1_FixedPoint32_ConcreteDivideAbortsIf"></a>


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteDivideAbortsIf">ConcreteDivideAbortsIf</a> {
    val: num;
    divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> divisor.value == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_divide_u64">spec_concrete_divide_u64</a>(val, divisor) &gt; <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_concrete_divide_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_divide_u64">spec_concrete_divide_u64</a>(val: num, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): num {
   (val &lt;&lt; 32) / divisor.value
}
</code></pre>



<a name="@Abstract_Semantics_2"></a>

### Abstract Semantics



<a name="0x1_FixedPoint32_DivideAbortsIf"></a>


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_DivideAbortsIf">DivideAbortsIf</a> {
    val: num;
    divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
    <b>aborts_if</b> divisor.value == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val, divisor) &gt; <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_divide_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val: num, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): num {
   <b>if</b> (divisor.value == 1)
       // 1.0
       val
   <b>else</b> <b>if</b> (divisor.value == 2)
       // 0.5
       val * 2
   <b>else</b>
       <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> + 1
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_rational"></a>

## Function `create_from_rational`

Create a fixed-point value from a rational number specified by its
numerator and denominator. Calling this function should be preferred
for using <code><a href="FixedPoint32.md#0x1_FixedPoint32_create_from_raw_value">Self::create_from_raw_value</a></code> which is also available.
This will abort if the denominator is zero. It will also
abort if the numerator is nonzero and the ratio is not in the range
2^-32 .. 2^32-1. When specifying decimal fractions, be careful about
rounding errors: if you round to display N digits after the decimal
point, you can use a denominator of 10^N to avoid numbers where the
very small imprecision in the binary representation could change the
rounding, e.g., 0.0125 will round down to 0.012 instead of up to 0.013.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    // If the denominator is zero, this will <b>abort</b>.
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    <b>assert</b>!(scaled_denominator != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>));
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    <b>assert</b>!(quotient != 0 || numerator == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
    // Return the quotient <b>as</b> a fixed-point number. We first need <b>to</b> check whether the cast
    // can succeed.
    <b>assert</b>!(quotient &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
    <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> { value: (quotient <b>as</b> u64) }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> [concrete] <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteCreateFromRationalAbortsIf">ConcreteCreateFromRationalAbortsIf</a>;
<b>ensures</b> [concrete] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_create_from_rational">spec_concrete_create_from_rational</a>(numerator, denominator);
<b>include</b> [abstract] <a href="FixedPoint32.md#0x1_FixedPoint32_CreateFromRationalAbortsIf">CreateFromRationalAbortsIf</a>;
<b>ensures</b> [abstract] result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator, denominator);
</code></pre>




<a name="0x1_FixedPoint32_ConcreteCreateFromRationalAbortsIf"></a>


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteCreateFromRationalAbortsIf">ConcreteCreateFromRationalAbortsIf</a> {
    numerator: u64;
    denominator: u64;
    <b>let</b> scaled_numerator = numerator &lt;&lt; 64;
    <b>let</b> scaled_denominator = denominator &lt;&lt; 32;
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    <b>aborts_if</b> scaled_denominator == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> quotient == 0 && scaled_numerator != 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> quotient &gt; <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_FixedPoint32_spec_concrete_create_from_rational"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_concrete_create_from_rational">spec_concrete_create_from_rational</a>(numerator: num, denominator: num): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
   <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>{value: (numerator &lt;&lt; 64) / (denominator &lt;&lt; 32)}
}
</code></pre>



<a name="@Abstract_Semantics_3"></a>

### Abstract Semantics



<a name="0x1_FixedPoint32_CreateFromRationalAbortsIf"></a>

This is currently identical to the concrete semantics.


<pre><code><b>schema</b> <a href="FixedPoint32.md#0x1_FixedPoint32_CreateFromRationalAbortsIf">CreateFromRationalAbortsIf</a> {
    <b>include</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ConcreteCreateFromRationalAbortsIf">ConcreteCreateFromRationalAbortsIf</a>;
}
</code></pre>


Abstract to either 0.5 or 1. This assumes validation of numerator and denominator has
succeeded.


<a name="0x1_FixedPoint32_spec_create_from_rational"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator: num, denominator: num): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
   <b>if</b> (numerator == denominator)
       // 1.0
       <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>{value: 1}
   <b>else</b>
       // 0.5
       <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>{value: 2}
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_raw_value"></a>

## Function `create_from_raw_value`

Create a fixedpoint value from a raw value.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> { value }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [concrete] result.value == value;
<b>ensures</b> [abstract] result.value == 2;
</code></pre>



</details>

<a name="0x1_FixedPoint32_get_raw_value"></a>

## Function `get_raw_value`

Accessor for the raw u64 value. Other less common operations, such as
adding or subtracting FixedPoint32 values, can be done using the raw
values directly.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    num.value
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_is_zero"></a>

## Function `is_zero`

Returns true if the ratio is zero.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_is_zero">is_zero</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_is_zero">is_zero</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): bool {
    num.value == 0
}
</code></pre>



</details>

<a name="@Module_Specification_4"></a>

## Module Specification




<pre><code><b>pragma</b> aborts_if_is_strict;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
