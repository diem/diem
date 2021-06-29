

<a name="@A_Root_Documentation_Template_0"></a>

# A Root Documentation Template


This document contains the description of multiple move scripts.

The script <code><a href="root_template_script3.md#yet_another">yet_another</a></code> is documented in its own file.

-  [Some Scripts](#@Some_Scripts_1)
    -  [Script `some`](#some)
-  [Other Scripts](#@Other_Scripts_2)
    -  [Script `other`](#other)
-  [Some other scripts from a module](#@Some_other_scripts_from_a_module_3)
    -  [Module `0x1::OneTypeOfScript`](#0x1_OneTypeOfScript)
        -  [Function `script1`](#0x1_OneTypeOfScript_script1)
        -  [Function `script2`](#0x1_OneTypeOfScript_script2)
    -  [Module `0x1::AnotherTypeOfScript`](#0x1_AnotherTypeOfScript)
        -  [Function `script3`](#0x1_AnotherTypeOfScript_script3)
        -  [Function `script4`](#0x1_AnotherTypeOfScript_script4)
-  [Index](#@Index_4)



<a name="@Some_Scripts_1"></a>

## Some Scripts



<a name="some"></a>

### Script `some`



<pre><code></code></pre>


This script does really nothing but just aborts.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#some">some</a>&lt;T&gt;(_account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="root.md#some">some</a>&lt;T&gt;(_account: signer) {
    <b>abort</b> 1
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>true</b> <b>with</b> 1;
</code></pre>



</details>



<a name="@Other_Scripts_2"></a>

## Other Scripts



<a name="other"></a>

### Script `other`



<pre><code></code></pre>


This script does also abort.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#other">other</a>&lt;T&gt;(_account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="root.md#other">other</a>&lt;T&gt;(_account: signer) {
    <b>abort</b> 2
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>true</b> <b>with</b> 2;
</code></pre>



</details>



<a name="@Some_other_scripts_from_a_module_3"></a>

## Some other scripts from a module



<a name="0x1_OneTypeOfScript"></a>

### Module `0x1::OneTypeOfScript`



<pre><code></code></pre>



<a name="0x1_OneTypeOfScript_script1"></a>

#### Function `script1`

This is a script


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_OneTypeOfScript_script1">script1</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_OneTypeOfScript_script1">script1</a>() {}
</code></pre>



</details>

<a name="0x1_OneTypeOfScript_script2"></a>

#### Function `script2`

This is another script


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_OneTypeOfScript_script2">script2</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_OneTypeOfScript_script2">script2</a>() {}
</code></pre>



</details>


This is another module full of script funs too:


<a name="0x1_AnotherTypeOfScript"></a>

### Module `0x1::AnotherTypeOfScript`



<pre><code></code></pre>



<a name="0x1_AnotherTypeOfScript_script3"></a>

#### Function `script3`

This is a script


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_AnotherTypeOfScript_script3">script3</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_AnotherTypeOfScript_script3">script3</a>() {}
</code></pre>



</details>

<a name="0x1_AnotherTypeOfScript_script4"></a>

#### Function `script4`

This is another script


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_AnotherTypeOfScript_script4">script4</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#0x1_AnotherTypeOfScript_script4">script4</a>() {}
</code></pre>



</details>



<a name="@Index_4"></a>

## Index


-  [`0x1::AnotherTypeOfScript`](root.md#0x1_AnotherTypeOfScript)
-  [`0x1::OneTypeOfScript`](root.md#0x1_OneTypeOfScript)
-  [`other`](root.md#other)
-  [`some`](root.md#some)
-  [`yet_another`](root_template_script3.md#yet_another)
