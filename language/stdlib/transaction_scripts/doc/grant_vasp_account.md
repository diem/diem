
<a name="SCRIPT"></a>

# Script `grant_vasp_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(root_vasp_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(root_vasp_addr: address) {
    <a href="../../modules/doc/association.md#0x0_Association_apply_for_privilege">Association::apply_for_privilege</a>&lt;<a href="../../modules/doc/vasp.md#0x0_VASP_CreationPrivilege">VASP::CreationPrivilege</a>&gt;();
    <a href="../../modules/doc/association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="../../modules/doc/vasp.md#0x0_VASP_CreationPrivilege">VASP::CreationPrivilege</a>&gt;(Transaction::sender());
    <a href="../../modules/doc/vasp.md#0x0_VASP_grant_vasp">VASP::grant_vasp</a>(root_vasp_addr);
}
</code></pre>



</details>
