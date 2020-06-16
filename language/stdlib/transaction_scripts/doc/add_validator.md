
<a name="SCRIPT"></a>

# Script `add_validator.move`

### Table of Contents

-  [Function `add_validator`](#SCRIPT_add_validator)



<a name="SCRIPT_add_validator"></a>

## Function `add_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address) {
    <b>let</b> assoc_root_role = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;AssociationRootRole&gt;(account);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(&assoc_root_role, validator_address);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, assoc_root_role);
}
</code></pre>



</details>
