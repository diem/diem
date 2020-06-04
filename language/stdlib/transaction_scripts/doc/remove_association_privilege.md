
<a name="SCRIPT"></a>

# Script `remove_association_privilege.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(account: &signer, addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Privilege&gt;(account: &signer, addr: address) {
    <a href="../../modules/doc/Association.md#0x0_Association_remove_privilege">Association::remove_privilege</a>&lt;Privilege&gt;(account, addr)
}
</code></pre>



</details>
