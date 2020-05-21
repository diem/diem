
<a name="0x0_Vector"></a>

# Module `0x0::Vector`

### Table of Contents

-  [Function `empty`](#0x0_Vector_empty)
-  [Function `length`](#0x0_Vector_length)
-  [Function `borrow`](#0x0_Vector_borrow)
-  [Function `push_back`](#0x0_Vector_push_back)
-  [Function `borrow_mut`](#0x0_Vector_borrow_mut)
-  [Function `pop_back`](#0x0_Vector_pop_back)
-  [Function `destroy_empty`](#0x0_Vector_destroy_empty)
-  [Function `swap`](#0x0_Vector_swap)
-  [Function `singleton`](#0x0_Vector_singleton)
-  [Function `reverse`](#0x0_Vector_reverse)
-  [Function `append`](#0x0_Vector_append)
-  [Function `is_empty`](#0x0_Vector_is_empty)
-  [Function `contains`](#0x0_Vector_contains)
-  [Function `index_of`](#0x0_Vector_index_of)
-  [Function `remove`](#0x0_Vector_remove)
-  [Function `swap_remove`](#0x0_Vector_swap_remove)
-  [Specification](#0x0_Vector_Specification)
    -  [Function `reverse`](#0x0_Vector_Specification_reverse)
    -  [Function `append`](#0x0_Vector_Specification_append)
    -  [Function `is_empty`](#0x0_Vector_Specification_is_empty)
    -  [Function `contains`](#0x0_Vector_Specification_contains)
    -  [Function `index_of`](#0x0_Vector_Specification_index_of)
    -  [Function `remove`](#0x0_Vector_Specification_remove)
    -  [Function `swap_remove`](#0x0_Vector_Specification_swap_remove)



<a name="0x0_Vector_empty"></a>

## Function `empty`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_empty">empty</a>&lt;Element&gt;(): vector&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_empty">empty</a>&lt;Element&gt;(): vector&lt;Element&gt;;
</code></pre>



</details>

<a name="0x0_Vector_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_length">length</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_length">length</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): u64;
</code></pre>



</details>

<a name="0x0_Vector_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_borrow">borrow</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, i: u64): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_borrow">borrow</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, i: u64): &Element;
</code></pre>



</details>

<a name="0x0_Vector_push_back"></a>

## Function `push_back`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_push_back">push_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, e: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_push_back">push_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, e: Element);
</code></pre>



</details>

<a name="0x0_Vector_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_borrow_mut">borrow_mut</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, idx: u64): &<b>mut</b> Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_borrow_mut">borrow_mut</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, idx: u64): &<b>mut</b> Element;
</code></pre>



</details>

<a name="0x0_Vector_pop_back"></a>

## Function `pop_back`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_pop_back">pop_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_pop_back">pop_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;): Element;
</code></pre>



</details>

<a name="0x0_Vector_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_destroy_empty">destroy_empty</a>&lt;Element&gt;(v: vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_destroy_empty">destroy_empty</a>&lt;Element&gt;(v: vector&lt;Element&gt;);
</code></pre>



</details>

<a name="0x0_Vector_swap"></a>

## Function `swap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_swap">swap</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64, j: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Vector_swap">swap</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64, j: u64);
</code></pre>



</details>

<a name="0x0_Vector_singleton"></a>

## Function `singleton`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_singleton">singleton</a>&lt;Element&gt;(e: Element): vector&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_singleton">singleton</a>&lt;Element&gt;(e: Element): vector&lt;Element&gt; {
    <b>let</b> v = <a href="#0x0_Vector_empty">empty</a>();
    <a href="#0x0_Vector_push_back">push_back</a>(&<b>mut</b> v, e);
    v
}
</code></pre>



</details>

<a name="0x0_Vector_reverse"></a>

## Function `reverse`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_reverse">reverse</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_reverse">reverse</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;) {
    <b>let</b> len = <a href="#0x0_Vector_length">length</a>(v);
    <b>if</b> (len == 0) <b>return</b> ();

    <b>let</b> front_index = 0;
    <b>let</b> back_index = len -1;
    <b>while</b> (front_index &lt; back_index) {
        <a href="#0x0_Vector_swap">swap</a>(v, front_index, back_index);
        front_index = front_index + 1;
        back_index = back_index - 1;
    }
}
</code></pre>



</details>

<a name="0x0_Vector_append"></a>

## Function `append`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_append">append</a>&lt;Element&gt;(lhs: &<b>mut</b> vector&lt;Element&gt;, other: vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_append">append</a>&lt;Element&gt;(lhs: &<b>mut</b> vector&lt;Element&gt;, other: vector&lt;Element&gt;) {
    <a href="#0x0_Vector_reverse">reverse</a>(&<b>mut</b> other);
    <b>while</b> (!<a href="#0x0_Vector_is_empty">is_empty</a>(&other)) <a href="#0x0_Vector_push_back">push_back</a>(lhs, <a href="#0x0_Vector_pop_back">pop_back</a>(&<b>mut</b> other));
    <a href="#0x0_Vector_destroy_empty">destroy_empty</a>(other);
}
</code></pre>



</details>

<a name="0x0_Vector_is_empty"></a>

## Function `is_empty`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_is_empty">is_empty</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_is_empty">is_empty</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): bool {
    <a href="#0x0_Vector_length">length</a>(v) == 0
}
</code></pre>



</details>

<a name="0x0_Vector_contains"></a>

## Function `contains`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_contains">contains</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_contains">contains</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): bool {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="#0x0_Vector_length">length</a>(v);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="#0x0_Vector_borrow">borrow</a>(v, i) == e) <b>return</b> <b>true</b>;
        i = i + 1;
    };
    <b>false</b>
}
</code></pre>



</details>

<a name="0x0_Vector_index_of"></a>

## Function `index_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_index_of">index_of</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): (bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_index_of">index_of</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): (bool, u64) {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="#0x0_Vector_length">length</a>(v);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="#0x0_Vector_borrow">borrow</a>(v, i) == e) <b>return</b> (<b>true</b>, i);
        i = i + 1;
    };
    (<b>false</b>, 0)
}
</code></pre>



</details>

<a name="0x0_Vector_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_remove">remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_remove">remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element {
    <b>let</b> len = <a href="#0x0_Vector_length">length</a>(v);
    // i out of bounds; <b>abort</b>
    <b>if</b> (i &gt;= len) <b>abort</b> 10;

    len = len - 1;
    <b>while</b> (i &lt; len) <a href="#0x0_Vector_swap">swap</a>(v, i, { i = i + 1; i });
    <a href="#0x0_Vector_pop_back">pop_back</a>(v)
}
</code></pre>



</details>

<a name="0x0_Vector_swap_remove"></a>

## Function `swap_remove`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_swap_remove">swap_remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_swap_remove">swap_remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element {
    <b>let</b> last_idx = <a href="#0x0_Vector_length">length</a>(v) - 1;
    <a href="#0x0_Vector_swap">swap</a>(v, i, last_idx);
    <a href="#0x0_Vector_pop_back">pop_back</a>(v)
}
</code></pre>



</details>

<a name="0x0_Vector_Specification"></a>

## Specification


<a name="0x0_Vector_Specification_reverse"></a>

### Function `reverse`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_reverse">reverse</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;)
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_append"></a>

### Function `append`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_append">append</a>&lt;Element&gt;(lhs: &<b>mut</b> vector&lt;Element&gt;, other: vector&lt;Element&gt;)
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_is_empty"></a>

### Function `is_empty`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_is_empty">is_empty</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): bool
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_contains"></a>

### Function `contains`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_contains">contains</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): bool
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_index_of"></a>

### Function `index_of`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_index_of">index_of</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): (bool, u64)
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_remove"></a>

### Function `remove`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_remove">remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>



<a name="0x0_Vector_Specification_swap_remove"></a>

### Function `swap_remove`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Vector_swap_remove">swap_remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>
