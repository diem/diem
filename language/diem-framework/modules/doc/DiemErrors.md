
<a name="0x1_DiemErrors"></a>

# Module `0x1::DiemErrors`

Module defining error codes used in Move aborts throughout the framework.

A <code>u64</code> error code is constructed from two values:

1. The *error category* which is encoded in the lower 8 bits of the code. Error categories are
declared in this module and are globally unique across the Diem framework. There is a limited
fixed set of predefined categories, and the framework is guaranteed to use those consistently.

2. The *error reason* which is encoded in the remaining 56 bits of the code. The reason is a unique
number relative to the module which raised the error and can be used to obtain more information about
the error at hand. It is mostly used for diagnosis purposes. Error reasons may change over time as the
framework evolves.

>TODO: determine what kind of stability guarantees we give about reasons/associated module.


-  [Constants](#@Constants_0)
-  [Function `make`](#0x1_DiemErrors_make)
-  [Function `requires_role`](#0x1_DiemErrors_requires_role)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemErrors_REQUIRES_ROLE"></a>

The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
which requires the signer to have the role of treasury compliance.


<pre><code><b>const</b> <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>: u8 = 3;
</code></pre>



<a name="0x1_DiemErrors_make"></a>

## Function `make`

A function to create an error from from a category and a reason.


<pre><code><b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_make">make</a>(category: u8, reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_make">make</a>(category: u8, reason: u64): u64 {
    (category <b>as</b> u64) + (reason &lt;&lt; 8)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>ensures</b> [concrete] result == category + (reason &lt;&lt; 8);
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == category;
</code></pre>



</details>

<a name="0x1_DiemErrors_requires_role"></a>

## Function `requires_role`



<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_role">requires_role</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemErrors.md#0x1_DiemErrors_requires_role">requires_role</a>(reason: u64): u64 { <a href="DiemErrors.md#0x1_DiemErrors_make">make</a>(<a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>, reason) }
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="DiemErrors.md#0x1_DiemErrors_REQUIRES_ROLE">REQUIRES_ROLE</a>;
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
