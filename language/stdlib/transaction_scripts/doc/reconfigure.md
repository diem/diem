
<a name="SCRIPT"></a>

# Script `reconfigure.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Update configs of all the validators and emit reconfiguration event.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer) {
    <b>let</b> assoc_root_role = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;AssociationRootRole&gt;(account);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_update_and_reconfigure">LibraSystem::update_and_reconfigure</a>(&assoc_root_role);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, assoc_root_role);
}
</code></pre>



</details>
