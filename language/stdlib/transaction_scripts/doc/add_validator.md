
<a name="SCRIPT"></a>

# Script `add_validator.move`

### Table of Contents

-  [Function `add_validator`](#SCRIPT_add_validator)



<a name="SCRIPT_add_validator"></a>

## Function `add_validator`

Add
<code>new_validator</code> to the pending validator set.
Fails if the
<code>new_validator</code> address is already in the validator set
or does not have a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code> resource stored at the address.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address) {
    <b>let</b> assoc_root_role = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;LibraRootRole&gt;(account);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(&assoc_root_role, validator_address);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, assoc_root_role);
}
</code></pre>



</details>
