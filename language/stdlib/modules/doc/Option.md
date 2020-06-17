
<a name="0x1_Option"></a>

# Module `0x1::Option`

### Table of Contents

-  [Struct `Option`](#0x1_Option_Option)
-  [Function `none`](#0x1_Option_none)
-  [Function `some`](#0x1_Option_some)
-  [Function `is_none`](#0x1_Option_is_none)
-  [Function `is_some`](#0x1_Option_is_some)
-  [Function `contains`](#0x1_Option_contains)
-  [Function `borrow`](#0x1_Option_borrow)
-  [Function `borrow_with_default`](#0x1_Option_borrow_with_default)
-  [Function `get_with_default`](#0x1_Option_get_with_default)
-  [Function `fill`](#0x1_Option_fill)
-  [Function `extract`](#0x1_Option_extract)
-  [Function `borrow_mut`](#0x1_Option_borrow_mut)
-  [Function `swap`](#0x1_Option_swap)
-  [Function `destroy_with_default`](#0x1_Option_destroy_with_default)
-  [Function `destroy_some`](#0x1_Option_destroy_some)
-  [Function `destroy_none`](#0x1_Option_destroy_none)
-  [Specification](#0x1_Option_Specification)
    -  [Struct `Option`](#0x1_Option_Specification_Option)
    -  [Function `none`](#0x1_Option_Specification_none)
    -  [Function `some`](#0x1_Option_Specification_some)
    -  [Function `is_none`](#0x1_Option_Specification_is_none)
    -  [Function `is_some`](#0x1_Option_Specification_is_some)
    -  [Function `contains`](#0x1_Option_Specification_contains)
    -  [Function `borrow`](#0x1_Option_Specification_borrow)
    -  [Function `borrow_with_default`](#0x1_Option_Specification_borrow_with_default)
    -  [Function `get_with_default`](#0x1_Option_Specification_get_with_default)
    -  [Function `fill`](#0x1_Option_Specification_fill)
    -  [Function `extract`](#0x1_Option_Specification_extract)
    -  [Function `borrow_mut`](#0x1_Option_Specification_borrow_mut)
    -  [Function `swap`](#0x1_Option_Specification_swap)
    -  [Function `destroy_with_default`](#0x1_Option_Specification_destroy_with_default)
    -  [Function `destroy_some`](#0x1_Option_Specification_destroy_some)
    -  [Function `destroy_none`](#0x1_Option_Specification_destroy_none)


This module defines the Option type and its methods to represent and handle an optional value.


<a name="0x1_Option_Option"></a>

## Struct `Option`



<pre><code><b>struct</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;
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

<a name="0x1_Option_none"></a>

## Function `none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_none">none</a>&lt;Element&gt;(): <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_none">none</a>&lt;Element&gt;(): <a href="#0x1_Option">Option</a>&lt;Element&gt; {
    <a href="#0x1_Option">Option</a> { vec: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x1_Option_some"></a>

## Function `some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_some">some</a>&lt;Element&gt;(e: Element): <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_some">some</a>&lt;Element&gt;(e: Element): <a href="#0x1_Option">Option</a>&lt;Element&gt; {
    <a href="#0x1_Option">Option</a> { vec: <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(e) }
}
</code></pre>



</details>

<a name="0x1_Option_is_none"></a>

## Function `is_none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_none">is_none</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_none">is_none</a>&lt;Element&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;): bool {
    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&t.vec)
}
</code></pre>



</details>

<a name="0x1_Option_is_some"></a>

## Function `is_some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_some">is_some</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_some">is_some</a>&lt;Element&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;): bool {
    !<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&t.vec)
}
</code></pre>



</details>

<a name="0x1_Option_contains"></a>

## Function `contains`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_contains">contains</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e_ref: &Element): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_contains">contains</a>&lt;Element&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;, e_ref: &Element): bool {
    <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&t.vec, e_ref)
}
</code></pre>



</details>

<a name="0x1_Option_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow">borrow</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow">borrow</a>&lt;Element&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;): &Element {
    <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&t.vec, 0)
}
</code></pre>



</details>

<a name="0x1_Option_borrow_with_default"></a>

## Function `borrow_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_with_default">borrow_with_default</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default_ref: &Element): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_with_default">borrow_with_default</a>&lt;Element&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;, default_ref: &Element): &Element {
    <b>let</b> vec_ref = &t.vec;
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(vec_ref)) default_ref
    <b>else</b> <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(vec_ref, 0)
}
</code></pre>



</details>

<a name="0x1_Option_get_with_default"></a>

## Function `get_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_get_with_default">get_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_get_with_default">get_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: &<a href="#0x1_Option">Option</a>&lt;Element&gt;, default: Element): Element {
    <b>let</b> vec_ref = &t.vec;
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(vec_ref)) default
    <b>else</b> *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(vec_ref, 0)
}
</code></pre>



</details>

<a name="0x1_Option_fill"></a>

## Function `fill`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_fill">fill</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_fill">fill</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;, e: Element) {
    <b>let</b> vec_ref = &<b>mut</b> t.vec;
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(vec_ref)) <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(vec_ref, e)
    <b>else</b> <b>abort</b>(99)
}
</code></pre>



</details>

<a name="0x1_Option_extract"></a>

## Function `extract`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_extract">extract</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_extract">extract</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;): Element {
    <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> t.vec)
}
</code></pre>



</details>

<a name="0x1_Option_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_mut">borrow_mut</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): &<b>mut</b> Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_mut">borrow_mut</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;): &<b>mut</b> Element {
    <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> t.vec, 0)
}
</code></pre>



</details>

<a name="0x1_Option_swap"></a>

## Function `swap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_swap">swap</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_swap">swap</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;, e: Element): Element {
    <b>let</b> vec_ref = &<b>mut</b> t.vec;
    <b>let</b> old_value = <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(vec_ref);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(vec_ref, e);
    old_value
}
</code></pre>



</details>

<a name="0x1_Option_destroy_with_default"></a>

## Function `destroy_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_with_default">destroy_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default: Element): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_with_default">destroy_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;, default: Element): Element {
    <b>let</b> <a href="#0x1_Option">Option</a> { vec } = t;
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&<b>mut</b> vec)) default
    <b>else</b> <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> vec)
}
</code></pre>



</details>

<a name="0x1_Option_destroy_some"></a>

## Function `destroy_some`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_some">destroy_some</a>&lt;Element&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_some">destroy_some</a>&lt;Element&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;): Element {
    <b>let</b> <a href="#0x1_Option">Option</a> { vec } = t;
    <b>let</b> elem = <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> vec);
    <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(vec);
    elem
}
</code></pre>



</details>

<a name="0x1_Option_destroy_none"></a>

## Function `destroy_none`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_none">destroy_none</a>&lt;Element&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_none">destroy_none</a>&lt;Element&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;) {
    <b>let</b> <a href="#0x1_Option">Option</a> { vec } = t;
    <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(vec)
}
</code></pre>



</details>

<a name="0x1_Option_Specification"></a>

## Specification


<a name="0x1_Option_Specification_Option"></a>

### Struct `Option`


<pre><code><b>struct</b> <a href="#0x1_Option">Option</a>&lt;Element&gt;
</code></pre>



<dl>
<dt>

<code>vec: vector&lt;Element&gt;</code>
</dt>
<dd>

</dd>
</dl>


The size of vector is always less than equal to 1
because it's 0 for "none" or 1 for "some".


<pre><code><b>invariant</b> len(vec) &lt;= 1;
</code></pre>



Return true iff t contains none.


<a name="0x1_Option_is_none_spec"></a>


<pre><code><b>define</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>&lt;Element&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;): bool {
    len(t.vec) == 0
}
</code></pre>


Return true iff t contains some.


<a name="0x1_Option_is_some_spec"></a>


<pre><code><b>define</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>&lt;Element&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;): bool {
    !<a href="#0x1_Option_is_none_spec">is_none_spec</a>(t)
}
</code></pre>


Return the value inside of t.


<a name="0x1_Option_value_inside"></a>


<pre><code><b>define</b> <a href="#0x1_Option_value_inside">value_inside</a>&lt;Element&gt;(t: <a href="#0x1_Option">Option</a>&lt;Element&gt;): Element {
    t.vec[0]
}
</code></pre>



<a name="0x1_Option_Specification_none"></a>

### Function `none`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_none">none</a>&lt;Element&gt;(): <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(result);
</code></pre>



<a name="0x1_Option_Specification_some"></a>

### Function `some`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_some">some</a>&lt;Element&gt;(e: Element): <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(result);
<b>ensures</b> <a href="#0x1_Option_value_inside">value_inside</a>(result) == e;
</code></pre>



<a name="0x1_Option_Specification_is_none"></a>

### Function `is_none`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_none">is_none</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
</code></pre>



<a name="0x1_Option_Specification_is_some"></a>

### Function `is_some`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_is_some">is_some</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t);
</code></pre>



<a name="0x1_Option_Specification_contains"></a>

### Function `contains`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_contains">contains</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e_ref: &Element): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == (<a href="#0x1_Option_is_some_spec">is_some_spec</a>(t) && <a href="#0x1_Option_value_inside">value_inside</a>(t) == e_ref);
</code></pre>



<a name="0x1_Option_Specification_borrow"></a>

### Function `borrow`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow">borrow</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): &Element
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
<b>ensures</b> result == <a href="#0x1_Option_value_inside">value_inside</a>(t);
</code></pre>



<a name="0x1_Option_Specification_borrow_with_default"></a>

### Function `borrow_with_default`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_with_default">borrow_with_default</a>&lt;Element&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default_ref: &Element): &Element
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t) ==&gt; result == default_ref;
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t) ==&gt; result == <a href="#0x1_Option_value_inside">value_inside</a>(t);
</code></pre>



<a name="0x1_Option_Specification_get_with_default"></a>

### Function `get_with_default`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_get_with_default">get_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: &<a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default: Element): Element
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t) ==&gt; result == default;
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t) ==&gt; result == <a href="#0x1_Option_value_inside">value_inside</a>(t);
</code></pre>



<a name="0x1_Option_Specification_fill"></a>

### Function `fill`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_fill">fill</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e: Element)
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t);
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t);
<b>ensures</b> <a href="#0x1_Option_value_inside">value_inside</a>(t) == e;
</code></pre>



<a name="0x1_Option_Specification_extract"></a>

### Function `extract`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_extract">extract</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): Element
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
<b>ensures</b> result == <a href="#0x1_Option_value_inside">value_inside</a>(<b>old</b>(t));
<b>ensures</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
</code></pre>



<a name="0x1_Option_Specification_borrow_mut"></a>

### Function `borrow_mut`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_borrow_mut">borrow_mut</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): &<b>mut</b> Element
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
<b>ensures</b> result == <a href="#0x1_Option_value_inside">value_inside</a>(t);
</code></pre>



<a name="0x1_Option_Specification_swap"></a>

### Function `swap`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_swap">swap</a>&lt;Element&gt;(t: &<b>mut</b> <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, e: Element): Element
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
<b>ensures</b> result == <a href="#0x1_Option_value_inside">value_inside</a>(<b>old</b>(t));
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t);
<b>ensures</b> <a href="#0x1_Option_value_inside">value_inside</a>(t) == e;
</code></pre>



<a name="0x1_Option_Specification_destroy_with_default"></a>

### Function `destroy_with_default`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_with_default">destroy_with_default</a>&lt;Element: <b>copyable</b>&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;, default: Element): Element
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(<b>old</b>(t)) ==&gt; result == default;
<b>ensures</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(<b>old</b>(t)) ==&gt; result == <a href="#0x1_Option_value_inside">value_inside</a>(<b>old</b>(t));
</code></pre>



<a name="0x1_Option_Specification_destroy_some"></a>

### Function `destroy_some`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_some">destroy_some</a>&lt;Element&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;): Element
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_none_spec">is_none_spec</a>(t);
<b>ensures</b> result == <a href="#0x1_Option_value_inside">value_inside</a>(<b>old</b>(t));
</code></pre>



<a name="0x1_Option_Specification_destroy_none"></a>

### Function `destroy_none`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Option_destroy_none">destroy_none</a>&lt;Element&gt;(t: <a href="#0x1_Option_Option">Option::Option</a>&lt;Element&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_Option_is_some_spec">is_some_spec</a>(t);
</code></pre>
