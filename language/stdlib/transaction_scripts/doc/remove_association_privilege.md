
<a name="SCRIPT"></a>

# Script `remove_association_privilege.move`

### Table of Contents

-  [Function `remove_association_privilege`](#SCRIPT_remove_association_privilege)



<a name="SCRIPT_remove_association_privilege"></a>

## Function `remove_association_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_remove_association_privilege">remove_association_privilege</a>&lt;Privilege&gt;(account: &signer, addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_remove_association_privilege">remove_association_privilege</a>&lt;Privilege&gt;(account: &signer, addr: address) {
    <a href="../../modules/doc/Association.md#0x1_Association_remove_privilege">Association::remove_privilege</a>&lt;Privilege&gt;(account, addr)
}
</code></pre>



</details>
