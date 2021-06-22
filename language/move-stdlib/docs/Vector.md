
<a name="0x1_Vector"></a>

# Module `0x1::Vector`

A variable-sized container that can hold any type. Indexing is 0-based, and
vectors are growable. This module has many native functions.
Verification of modules that use this one uses model functions that are implemented
directly in Boogie. The specification language has built-in functions operations such
as <code>singleton_vector</code>. There are some helper functions defined here for specifications in other
modules as well.

>Note: We did not verify most of the
Move functions here because many have loops, requiring loop invariants to prove, and
the return on investment didn't seem worth it for these simple functions.


-  [Constants](#@Constants_0)
-  [Function `empty`](#0x1_Vector_empty)
-  [Function `length`](#0x1_Vector_length)
-  [Function `borrow`](#0x1_Vector_borrow)
-  [Function `push_back`](#0x1_Vector_push_back)
-  [Function `borrow_mut`](#0x1_Vector_borrow_mut)
-  [Function `pop_back`](#0x1_Vector_pop_back)
-  [Function `destroy_empty`](#0x1_Vector_destroy_empty)
-  [Function `swap`](#0x1_Vector_swap)
-  [Function `singleton`](#0x1_Vector_singleton)
-  [Function `reverse`](#0x1_Vector_reverse)
-  [Function `append`](#0x1_Vector_append)
-  [Function `is_empty`](#0x1_Vector_is_empty)
-  [Function `contains`](#0x1_Vector_contains)
-  [Function `index_of`](#0x1_Vector_index_of)
-  [Function `remove`](#0x1_Vector_remove)
-  [Function `swap_remove`](#0x1_Vector_swap_remove)
-  [Module Specification](#@Module_Specification_1)
    -  [Helper Functions](#@Helper_Functions_2)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_Vector_EINDEX_OUT_OF_BOUNDS"></a>

The index into the vector is out of bounds


<pre><code><b>const</b> <a href="Vector.md#0x1_Vector_EINDEX_OUT_OF_BOUNDS">EINDEX_OUT_OF_BOUNDS</a>: u64 = 0;
</code></pre>



<a name="0x1_Vector_empty"></a>

## Function `empty`

Create an empty vector.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_empty">empty</a>&lt;Element&gt;(): vector&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_empty">empty</a>&lt;Element&gt;(): vector&lt;Element&gt;;
</code></pre>



</details>

<a name="0x1_Vector_length"></a>

## Function `length`

Return the length of the vector.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_length">length</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_length">length</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): u64;
</code></pre>



</details>

<a name="0x1_Vector_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the <code>i</code>th element of the vector <code>v</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_borrow">borrow</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, i: u64): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_borrow">borrow</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, i: u64): &Element;
</code></pre>



</details>

<a name="0x1_Vector_push_back"></a>

## Function `push_back`

Add element <code>e</code> to the end of the vector <code>v</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_push_back">push_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, e: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_push_back">push_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, e: Element);
</code></pre>



</details>

<a name="0x1_Vector_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th element in the vector <code>v</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_borrow_mut">borrow_mut</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): &<b>mut</b> Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_borrow_mut">borrow_mut</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): &<b>mut</b> Element;
</code></pre>



</details>

<a name="0x1_Vector_pop_back"></a>

## Function `pop_back`

Pop an element from the end of vector <code>v</code>.
Aborts if <code>v</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_pop_back">pop_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_pop_back">pop_back</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;): Element;
</code></pre>



</details>

<a name="0x1_Vector_destroy_empty"></a>

## Function `destroy_empty`

Destroy the vector <code>v</code>.
Aborts if <code>v</code> is not empty.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_destroy_empty">destroy_empty</a>&lt;Element&gt;(v: vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_destroy_empty">destroy_empty</a>&lt;Element&gt;(v: vector&lt;Element&gt;);
</code></pre>



</details>

<a name="0x1_Vector_swap"></a>

## Function `swap`

Swaps the elements at the <code>i</code>th and <code>j</code>th indices in the vector <code>v</code>.
Aborts if <code>i</code>or <code>j</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_swap">swap</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64, j: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_swap">swap</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64, j: u64);
</code></pre>



</details>

<a name="0x1_Vector_singleton"></a>

## Function `singleton`

Return an vector of size one containing element <code>e</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_singleton">singleton</a>&lt;Element&gt;(e: Element): vector&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_singleton">singleton</a>&lt;Element&gt;(e: Element): vector&lt;Element&gt; {
    <b>let</b> v = <a href="Vector.md#0x1_Vector_empty">empty</a>();
    <a href="Vector.md#0x1_Vector_push_back">push_back</a>(&<b>mut</b> v, e);
    v
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == vec(e);
</code></pre>



</details>

<a name="0x1_Vector_reverse"></a>

## Function `reverse`

Reverses the order of the elements in the vector <code>v</code> in place.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_reverse">reverse</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_reverse">reverse</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;) {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">length</a>(v);
    <b>if</b> (len == 0) <b>return</b> ();

    <b>let</b> front_index = 0;
    <b>let</b> back_index = len -1;
    <b>while</b> (front_index &lt; back_index) {
        <a href="Vector.md#0x1_Vector_swap">swap</a>(v, front_index, back_index);
        front_index = front_index + 1;
        back_index = back_index - 1;
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_append"></a>

## Function `append`

Pushes all of the elements of the <code>other</code> vector into the <code>lhs</code> vector.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_append">append</a>&lt;Element&gt;(lhs: &<b>mut</b> vector&lt;Element&gt;, other: vector&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_append">append</a>&lt;Element&gt;(lhs: &<b>mut</b> vector&lt;Element&gt;, other: vector&lt;Element&gt;) {
    <a href="Vector.md#0x1_Vector_reverse">reverse</a>(&<b>mut</b> other);
    <b>while</b> (!<a href="Vector.md#0x1_Vector_is_empty">is_empty</a>(&other)) <a href="Vector.md#0x1_Vector_push_back">push_back</a>(lhs, <a href="Vector.md#0x1_Vector_pop_back">pop_back</a>(&<b>mut</b> other));
    <a href="Vector.md#0x1_Vector_destroy_empty">destroy_empty</a>(other);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_is_empty"></a>

## Function `is_empty`

Return <code><b>true</b></code> if the vector <code>v</code> has no elements and <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_is_empty">is_empty</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_is_empty">is_empty</a>&lt;Element&gt;(v: &vector&lt;Element&gt;): bool {
    <a href="Vector.md#0x1_Vector_length">length</a>(v) == 0
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_contains"></a>

## Function `contains`

Return true if <code>e</code> is in the vector <code>v</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_contains">contains</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_contains">contains</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): bool {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">length</a>(v);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="Vector.md#0x1_Vector_borrow">borrow</a>(v, i) == e) <b>return</b> <b>true</b>;
        i = i + 1;
    };
    <b>false</b>
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_index_of"></a>

## Function `index_of`

Return <code>(<b>true</b>, i)</code> if <code>e</code> is in the vector <code>v</code> at index <code>i</code>.
Otherwise, returns <code>(<b>false</b>, 0)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_index_of">index_of</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): (bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_index_of">index_of</a>&lt;Element&gt;(v: &vector&lt;Element&gt;, e: &Element): (bool, u64) {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">length</a>(v);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="Vector.md#0x1_Vector_borrow">borrow</a>(v, i) == e) <b>return</b> (<b>true</b>, i);
        i = i + 1;
    };
    (<b>false</b>, 0)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_remove"></a>

## Function `remove`

Remove the <code>i</code>th element of the vector <code>v</code>, shifting all subsequent elements.
This is O(n) and preserves ordering of elements in the vector.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_remove">remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_remove">remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">length</a>(v);
    // i out of bounds; <b>abort</b>
    <b>if</b> (i &gt;= len) <b>abort</b> <a href="Vector.md#0x1_Vector_EINDEX_OUT_OF_BOUNDS">EINDEX_OUT_OF_BOUNDS</a>;

    len = len - 1;
    <b>while</b> (i &lt; len) <a href="Vector.md#0x1_Vector_swap">swap</a>(v, i, { i = i + 1; i });
    <a href="Vector.md#0x1_Vector_pop_back">pop_back</a>(v)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="0x1_Vector_swap_remove"></a>

## Function `swap_remove`

Swap the <code>i</code>th element of the vector <code>v</code> with the last element and then pop the vector.
This is O(1), but does not preserve ordering of elements in the vector.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_swap_remove">swap_remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Vector.md#0x1_Vector_swap_remove">swap_remove</a>&lt;Element&gt;(v: &<b>mut</b> vector&lt;Element&gt;, i: u64): Element {
    <b>assert</b>(!<a href="Vector.md#0x1_Vector_is_empty">is_empty</a>(v), <a href="Vector.md#0x1_Vector_EINDEX_OUT_OF_BOUNDS">EINDEX_OUT_OF_BOUNDS</a>);
    <b>let</b> last_idx = <a href="Vector.md#0x1_Vector_length">length</a>(v) - 1;
    <a href="Vector.md#0x1_Vector_swap">swap</a>(v, i, last_idx);
    <a href="Vector.md#0x1_Vector_pop_back">pop_back</a>(v)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Helper_Functions_2"></a>

### Helper Functions


Check if <code>v1</code> is equal to the result of adding <code>e</code> at the end of <code>v2</code>


<a name="0x1_Vector_eq_push_back"></a>


<pre><code><b>fun</b> <a href="Vector.md#0x1_Vector_eq_push_back">eq_push_back</a>&lt;Element&gt;(v1: vector&lt;Element&gt;, v2: vector&lt;Element&gt;, e: Element): bool {
    len(v1) == len(v2) + 1 &&
    v1[len(v1)-1] == e &&
    v1[0..len(v1)-1] == v2[0..len(v2)]
}
</code></pre>


Check if <code>v</code> is equal to the result of concatenating <code>v1</code> and <code>v2</code>


<a name="0x1_Vector_eq_append"></a>


<pre><code><b>fun</b> <a href="Vector.md#0x1_Vector_eq_append">eq_append</a>&lt;Element&gt;(v: vector&lt;Element&gt;, v1: vector&lt;Element&gt;, v2: vector&lt;Element&gt;): bool {
    len(v) == len(v1) + len(v2) &&
    v[0..len(v1)] == v1 &&
    v[len(v1)..len(v)] == v2
}
</code></pre>


Check <code>v1</code> is equal to the result of removing the first element of <code>v2</code>


<a name="0x1_Vector_eq_pop_front"></a>


<pre><code><b>fun</b> <a href="Vector.md#0x1_Vector_eq_pop_front">eq_pop_front</a>&lt;Element&gt;(v1: vector&lt;Element&gt;, v2: vector&lt;Element&gt;): bool {
    len(v1) + 1 == len(v2) &&
    v1 == v2[1..len(v2)]
}
</code></pre>


Check that <code>v1</code> is equal to the result of removing the element at index <code>i</code> from <code>v2</code>.


<a name="0x1_Vector_eq_remove_elem_at_index"></a>


<pre><code><b>fun</b> <a href="Vector.md#0x1_Vector_eq_remove_elem_at_index">eq_remove_elem_at_index</a>&lt;Element&gt;(i: u64, v1: vector&lt;Element&gt;, v2: vector&lt;Element&gt;): bool {
    len(v1) + 1 == len(v2) &&
    v1[0..i] == v2[0..i] &&
    v1[i..len(v1)] == v2[i + 1..len(v2)]
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
