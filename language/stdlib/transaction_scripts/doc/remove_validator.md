
<a name="SCRIPT"></a>

# Script `remove_validator.move`

### Table of Contents

-  [Function `remove_validator`](#SCRIPT_remove_validator)



<a name="SCRIPT_remove_validator"></a>

## Function `remove_validator`

Adding
<code>to_remove</code> to the set of pending validator removals. Fails if
the
<code>to_remove</code> address is already in the validator set or already in the pending removals.
Callable by Validator's operator.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_remove_validator">remove_validator</a>(account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_remove_validator">remove_validator</a>(account: &signer, validator_address: address) {
    <b>let</b> assoc_root_role = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;LibraRootRole&gt;(account);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_remove_validator">LibraSystem::remove_validator</a>(&assoc_root_role, validator_address);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, assoc_root_role);
}
</code></pre>



</details>
