
<a name="0x1_DiemErrors"></a>

# Module `0x1::DiemErrors`

Module defining error codes used in Move aborts throughout the Diem Framework.

A <code>u64</code> error code is constructed from two values:

1. The *error category* which is encoded in the lower 8 bits of the code. Error categories are
declared in this module and are globally unique across the Diem framework. There is a limited
fixed set of predefined categories, and the framework is guaranteed to use those consistently.

2. The *error reason* which is encoded in the remaining 56 bits of the code. The reason is a unique
number relative to the module which raised the error and can be used to obtain more information about
the error at hand. It is mostly used for diagnosis purposes. Error reasons may change over time as the
framework evolves.


-  [Constants](#@Constants_0)
-  [Function `requires_role`](#0x1_DiemErrors_requires_role)
-  [Function `requires_capability`](#0x1_DiemErrors_requires_capability)


<pre><code><b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemErrors_REQUIRES_CAPABILITY"></a>

The signer of a transaction does not have a required capability.


<pre><code><b>const</b> <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>: u8 = 4;
</code></pre>



<a name="0x1_DiemErrors_REQUIRES_ROLE"></a>

The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
which requires the signer to have the role of treasury compliance.


<pre><code><b>const</b> <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>: u8 = 3;
</code></pre>



<a name="0x1_DiemErrors_requires_role"></a>

## Function `requires_role`



<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_role">requires_role</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_role">requires_role</a>(reason: u64): u64 { <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_make">Errors::make</a>(<a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>, reason) }
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>;
</code></pre>



</details>

<a name="0x1_DiemErrors_requires_capability"></a>

## Function `requires_capability`



<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_capability">requires_capability</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_capability">requires_capability</a>(reason: u64): u64 { <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_make">Errors::make</a>(<a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>, reason) }
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>;
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
