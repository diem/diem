

<a name="@A_Root_Documentation_Template_0"></a>

# A Root Documentation Template


This document contains the description of multiple move scripts.

The script <code><a href="root_template_script3.md#yet_another">yet_another</a></code> is documented in its own file.

-  [Some Scripts](#@Some_Scripts_1)
    -  [Script `some`](#some)
-  [Other Scripts](#@Other_Scripts_2)
    -  [Script `other`](#other)
-  [Index](#@Index_3)



<a name="@Some_Scripts_1"></a>

## Some Scripts



<a name="some"></a>

### Script `some`



<pre><code></code></pre>


This script does really nothing but just aborts.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#some">some</a>&lt;T&gt;(_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="root.md#some">some</a>&lt;T&gt;(_account: &signer) {
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


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="root.md#other">other</a>&lt;T&gt;(_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="root.md#other">other</a>&lt;T&gt;(_account: &signer) {
    <b>abort</b> 2
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>true</b> <b>with</b> 2;
</code></pre>



</details>



<a name="@Index_3"></a>

## Index


-  [`other`](root.md#other)
-  [`some`](root.md#some)
-  [`yet_another`](root_template_script3.md#yet_another)
