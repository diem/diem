
<a name="0x0_Option"></a>

# Module `0x0::Option`

### Table of Contents

-  [Struct `T`](#0x0_Option_T)
-  [Function `none`](#0x0_Option_none)
-  [Function `some`](#0x0_Option_some)
-  [Function `is_none`](#0x0_Option_is_none)
-  [Function `is_some`](#0x0_Option_is_some)
-  [Function `contains`](#0x0_Option_contains)
-  [Function `borrow`](#0x0_Option_borrow)
-  [Function `borrow_with_default`](#0x0_Option_borrow_with_default)
-  [Function `get_with_default`](#0x0_Option_get_with_default)
-  [Function `fill`](#0x0_Option_fill)
-  [Function `extract`](#0x0_Option_extract)
-  [Function `borrow_mut`](#0x0_Option_borrow_mut)
-  [Function `swap`](#0x0_Option_swap)
-  [Function `destroy_with_default`](#0x0_Option_destroy_with_default)
-  [Function `destroy_some`](#0x0_Option_destroy_some)
-  [Function `destroy_none`](#0x0_Option_destroy_none)



<a name="0x0_Option_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_Option_T">T</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>vec: vector&lt;Element&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Option_none"></a>

## Function `none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_none">none</a>&lt;Element&gt;(): <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_none">none</a>&lt;Element&gt;(): <a href="#0x0_Option_T">T</a>&lt;Element&gt; {
    <a href="#0x0_Option_T">T</a> { vec: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x0_Option_some"></a>

## Function `some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_some">some</a>&lt;Element&gt;(e: Element): <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_some">some</a>&lt;Element&gt;(e: Element): <a href="#0x0_Option_T">T</a>&lt;Element&gt; {
    <a href="#0x0_Option_T">T</a> { vec: <a href="vector.md#0x0_Vector_singleton">Vector::singleton</a>(e) }
}
</code></pre>



</details>

<a name="0x0_Option_is_none"></a>

## Function `is_none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_is_none">is_none</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_is_none">is_none</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;): bool {
    <a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&t.vec)
}
</code></pre>



</details>

<a name="0x0_Option_is_some"></a>

## Function `is_some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_is_some">is_some</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_is_some">is_some</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;): bool {
    !<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&t.vec)
}
</code></pre>



</details>

<a name="0x0_Option_contains"></a>

## Function `contains`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_contains">contains</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, e_ref: &Element): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_contains">contains</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;, e_ref: &Element): bool {
    <a href="vector.md#0x0_Vector_contains">Vector::contains</a>(&t.vec, e_ref)
}
</code></pre>



</details>

<a name="0x0_Option_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow">borrow</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow">borrow</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;): &Element {
    <a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&t.vec, 0)
}
</code></pre>



</details>

<a name="0x0_Option_borrow_with_default"></a>

## Function `borrow_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow_with_default">borrow_with_default</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, default_ref: &Element): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow_with_default">borrow_with_default</a>&lt;Element&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;, default_ref: &Element): &Element {
    <b>let</b> vec_ref = &t.vec;
    <b>if</b> (<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(vec_ref)) default_ref
    <b>else</b> <a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(vec_ref, 0)
}
</code></pre>



</details>

<a name="0x0_Option_get_with_default"></a>

## Function `get_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_get_with_default">get_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: &<a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, default: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_get_with_default">get_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: &<a href="#0x0_Option_T">T</a>&lt;Element&gt;, default: Element): Element {
    <b>let</b> vec_ref = &t.vec;
    <b>if</b> (<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(vec_ref)) default
    <b>else</b> *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(vec_ref, 0)
}
</code></pre>



</details>

<a name="0x0_Option_fill"></a>

## Function `fill`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_fill">fill</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, e: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_fill">fill</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">T</a>&lt;Element&gt;, e: Element) {
    <b>let</b> vec_ref = &<b>mut</b> t.vec;
    <b>if</b> (<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(vec_ref)) <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(vec_ref, e)
    <b>else</b> <b>abort</b>(99)
}
</code></pre>



</details>

<a name="0x0_Option_extract"></a>

## Function `extract`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_extract">extract</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_extract">extract</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">T</a>&lt;Element&gt;): Element {
    <a href="vector.md#0x0_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> t.vec)
}
</code></pre>



</details>

<a name="0x0_Option_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow_mut">borrow_mut</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): &<b>mut</b> Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_borrow_mut">borrow_mut</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">T</a>&lt;Element&gt;): &<b>mut</b> Element {
    <a href="vector.md#0x0_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> t.vec, 0)
}
</code></pre>



</details>

<a name="0x0_Option_swap"></a>

## Function `swap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_swap">swap</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, e: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_swap">swap</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x0_Option_T">T</a>&lt;Element&gt;, e: Element): Element {
    <b>let</b> vec_ref = &<b>mut</b> t.vec;
    <b>let</b> old_value = <a href="vector.md#0x0_Vector_pop_back">Vector::pop_back</a>(vec_ref);
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(vec_ref, e);
    old_value
}
</code></pre>



</details>

<a name="0x0_Option_destroy_with_default"></a>

## Function `destroy_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_with_default">destroy_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;, default: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_with_default">destroy_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: <a href="#0x0_Option_T">T</a>&lt;Element&gt;, default: Element): Element {
    <b>let</b> <a href="#0x0_Option_T">T</a> { vec } = t;
    <b>if</b> (<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&<b>mut</b> vec)) default
    <b>else</b> <a href="vector.md#0x0_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> vec)
}
</code></pre>



</details>

<a name="0x0_Option_destroy_some"></a>

## Function `destroy_some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_some">destroy_some</a>&lt;Element&gt;(t: <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_some">destroy_some</a>&lt;Element&gt;(t: <a href="#0x0_Option_T">T</a>&lt;Element&gt;): Element {
    <b>let</b> <a href="#0x0_Option_T">T</a> { vec } = t;
    <b>let</b> elem = <a href="vector.md#0x0_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> vec);
    <a href="vector.md#0x0_Vector_destroy_empty">Vector::destroy_empty</a>(vec);
    elem
}
</code></pre>



</details>

<a name="0x0_Option_destroy_none"></a>

## Function `destroy_none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_none">destroy_none</a>&lt;Element&gt;(t: <a href="#0x0_Option_T">Option::T</a>&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Option_destroy_none">destroy_none</a>&lt;Element&gt;(t: <a href="#0x0_Option_T">T</a>&lt;Element&gt;) {
    <b>let</b> <a href="#0x0_Option_T">T</a> { vec } = t;
    <a href="vector.md#0x0_Vector_destroy_empty">Vector::destroy_empty</a>(vec)
}
</code></pre>



</details>
