
<a name="0x1_BCSSet"></a>

# Module `0x1::BCSSet`



-  [Struct `Set`](#0x1_BCSSet_Set)
-  [Function `empty`](#0x1_BCSSet_empty)
-  [Function `insert_or_return`](#0x1_BCSSet_insert_or_return)
-  [Function `insert`](#0x1_BCSSet_insert)
-  [Function `is_mem`](#0x1_BCSSet_is_mem)
-  [Function `borrow_index`](#0x1_BCSSet_borrow_index)
-  [Function `borrow`](#0x1_BCSSet_borrow)
-  [Function `size`](#0x1_BCSSet_size)
-  [Function `elements`](#0x1_BCSSet_elements)
-  [Function `find`](#0x1_BCSSet_find)


<pre><code><b>use</b> <a href="BCS.md#0x1_BCS">0x1::BCS</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_BCSSet_Set"></a>

## Struct `Set`

A set of elements ordered by their BCS serialization.
Note that this is often different than the natural ordering on elements
(e.g., for addresses)--see Std::Compare for a detailed explanation


<pre><code><b>struct</b> <a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt; has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>elements: vector&lt;Elem&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_BCSSet_empty"></a>

## Function `empty`

Create an empty set


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_empty">empty</a>&lt;Elem&gt;(): <a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_empty">empty</a>&lt;Elem&gt;(): <a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt; {
    <a href="BCSSet.md#0x1_BCSSet_Set">Set</a> { elements: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x1_BCSSet_insert_or_return"></a>

## Function `insert_or_return`

Add <code>e</code> to <code>s</code>. Return <code>e</code> wrapped in an <code><a href="Option.md#0x1_Option">Option</a></code> if it is already
present in <code>t</code>and <code>None</code> otherwise


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_insert_or_return">insert_or_return</a>&lt;Elem&gt;(s: &<b>mut</b> <a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, e: Elem): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;Elem&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_insert_or_return">insert_or_return</a>&lt;Elem&gt;(s: &<b>mut</b> <a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, e: Elem): <a href="Option.md#0x1_Option">Option</a>&lt;Elem&gt; {
    <b>let</b> (insert_at, found) = <a href="BCSSet.md#0x1_BCSSet_find">find</a>(s, &e);
    <b>if</b> (found) {
        <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(e)
    };
    // <b>else</b>, not found--insert
    <b>let</b> elements = &<b>mut</b> s.elements;
    // TODO: Vector::insert(index, e) would be useful here.
    <b>let</b> i = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(elements);
    // add e <b>to</b> the end and then <b>move</b> it  <b>to</b> the left until we hit `insert_at`
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(elements, e);
    <b>while</b> (i &gt; insert_at) {
        <a href="Vector.md#0x1_Vector_swap">Vector::swap</a>(elements, i, i - 1);
        i = i - 1;
    };
    <a href="Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x1_BCSSet_insert"></a>

## Function `insert`

Add <code>e</code> to <code>s</code> Return false if <code>e</code> was already present in <code>t</code> and <code><b>true</b></code> otherwise


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_insert">insert</a>&lt;Elem: drop&gt;(s: &<b>mut</b> <a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, e: Elem): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_insert">insert</a>&lt;Elem: drop&gt;(s: &<b>mut</b> <a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, e: Elem): bool {
    <b>let</b> e_opt = <a href="BCSSet.md#0x1_BCSSet_insert_or_return">insert_or_return</a>(s, e);
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&e_opt)
}
</code></pre>



</details>

<a name="0x1_BCSSet_is_mem"></a>

## Function `is_mem`

Return true if <code>t</code> contains <code>e</code>, false otherwise


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_is_mem">is_mem</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, e: &Elem): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_is_mem">is_mem</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, e: &Elem): bool {
    <b>let</b> (_index, found) = <a href="BCSSet.md#0x1_BCSSet_find">find</a>(s, e);
    found
}
</code></pre>



</details>

<a name="0x1_BCSSet_borrow_index"></a>

## Function `borrow_index`

If <code>t</code> contains <code>e</code>, return<code>e's</code> index wrapped in an <code><a href="Option.md#0x1_Option">Option</a></code>.
Otherwise, return <code>None</code>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_borrow_index">borrow_index</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, e: &Elem): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_borrow_index">borrow_index</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, e: &Elem): <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> (index, found) = <a href="BCSSet.md#0x1_BCSSet_find">find</a>(s, e);
    <b>if</b> (found) {
        <a href="Option.md#0x1_Option_some">Option::some</a>(index)
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>()
    }
}
</code></pre>



</details>

<a name="0x1_BCSSet_borrow"></a>

## Function `borrow`

Borrow the <code>index</code>th element of <code>s</code>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_borrow">borrow</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, index: u64): &Elem
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_borrow">borrow</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, index: u64): &Elem {
    <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&s.elements, index)
}
</code></pre>



</details>

<a name="0x1_BCSSet_size"></a>

## Function `size`

Return the cardinality of <code>s</code>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_size">size</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_size">size</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;): u64 {
   <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&s.elements)
}
</code></pre>



</details>

<a name="0x1_BCSSet_elements"></a>

## Function `elements`

Return the backing array of this Set


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_elements">elements</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;): &vector&lt;Elem&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BCSSet.md#0x1_BCSSet_elements">elements</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;): &vector&lt;Elem&gt; {
    &s.elements
}
</code></pre>



</details>

<a name="0x1_BCSSet_find"></a>

## Function `find`



<pre><code><b>fun</b> <a href="BCSSet.md#0x1_BCSSet_find">find</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">BCSSet::Set</a>&lt;Elem&gt;, e: &Elem): (u64, bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="BCSSet.md#0x1_BCSSet_find">find</a>&lt;Elem&gt;(s: &<a href="BCSSet.md#0x1_BCSSet_Set">Set</a>&lt;Elem&gt;, e: &Elem): (u64, bool) {
    <b>let</b> e_bcs = <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(e);
    <b>let</b> elements = &s.elements;

    <b>let</b> left = 0;
    <b>let</b> len =  <a href="Vector.md#0x1_Vector_length">Vector::length</a>(elements);
    <b>if</b> (len == 0) {
        <b>return</b> (0, <b>false</b>)
    };
    // <b>use</b> binary search <b>to</b> locate `e` (<b>if</b> it <b>exists</b>)
    <b>let</b> right = len - 1;
    <b>while</b> (left &lt;= right) {
        <b>let</b> mid = (left + right) / 2;
        <b>let</b> cmp = <a href="BCS.md#0x1_BCS_cmp_bytes">BCS::cmp_bytes</a>(&<a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(elements, mid)), &e_bcs);
        <b>if</b> (cmp == 0u8) {
            <b>return</b> (mid, <b>true</b>)
        } <b>else</b> <b>if</b> (cmp == 1u8) {
            left = mid + 1
        } <b>else</b> { // cmp == 2u8
            <b>if</b> (mid == 0) {
                <b>return</b> (0, <b>false</b>)
            };
            <b>assert</b>(mid != 0, 88);
            right = mid -1
        }
    };
    (left, <b>false</b>)
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
