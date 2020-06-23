
<a name="SCRIPT"></a>

# Script `set_vasp_expiration.move`

### Table of Contents

-  [Function `set_vasp_expiration`](#SCRIPT_set_vasp_expiration)



<a name="SCRIPT_set_vasp_expiration"></a>

## Function `set_vasp_expiration`

Set the expiration time of the parent vasp account at
<code>parent_vasp_address</code>
to expire at
<code>new_expiration_date</code> specified in microseconds according to
blockchain time. This can only be invoked by the Libra root account.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_vasp_expiration">set_vasp_expiration</a>(association: &signer, parent_vasp_address: address, new_expiration_date: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_vasp_expiration">set_vasp_expiration</a>(
    association: &signer,
    parent_vasp_address: address,
    new_expiration_date: u64,
) {
    <b>let</b> assoc_root_capability = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;LibraRootRole&gt;(association);
    <a href="../../modules/doc/VASP.md#0x1_VASP_set_vasp_expiration">VASP::set_vasp_expiration</a>(&assoc_root_capability, parent_vasp_address, new_expiration_date);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(association, assoc_root_capability);
}
</code></pre>



</details>
