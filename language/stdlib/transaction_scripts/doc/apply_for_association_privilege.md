
<a name="SCRIPT"></a>

# Script `apply_for_association_privilege.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Privilege&gt;() {
    <a href="../../modules/doc/association.md#0x0_Association_apply_for_privilege">Association::apply_for_privilege</a>&lt;Privilege&gt;();
}
</code></pre>



</details>
