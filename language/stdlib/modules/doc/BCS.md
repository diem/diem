
<a name="0x1_BCS"></a>

# Module `0x1::BCS`

Utility for converting a Move value to its binary representation in BCS (Binary Canonical
Serialization). BCS is the binary encoding for Move resources and other non-module values
published on-chain. See https://github.com/diem/diem/tree/master/common/bcs for more
details on BCS.


-  [Function `to_bytes`](#0x1_BCS_to_bytes)
-  [Module Specification](#@Module_Specification_0)


<pre><code></code></pre>



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

<a name="@Module_Specification_0"></a>

## Module Specification



Native function which is defined in the prover's prelude.


<a name="0x1_BCS_serialize"></a>


<pre><code><b>native</b> <b>define</b> <a href="BCS.md#0x1_BCS_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions
