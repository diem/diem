
<a name="0x0_LCS"></a>

# Module `0x0::LCS`

### Table of Contents

-  [Function `to_bytes`](#0x0_LCS_to_bytes)
-  [Specification](#0x0_LCS_Specification)



<a name="0x0_LCS_to_bytes"></a>

## Function `to_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_LCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x0_LCS_Specification"></a>

## Specification



<a name="0x0_LCS_serialize"></a>


<pre><code><b>native</b> <b>define</b> <a href="#0x0_LCS_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>
