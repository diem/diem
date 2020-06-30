
<a name="SCRIPT"></a>

# Script `update_minting_ability.move`

### Table of Contents

-  [Function `update_minting_ability`](#SCRIPT_update_minting_ability)



<a name="SCRIPT_update_minting_ability"></a>

## Function `update_minting_ability`

Allows--true--or disallows--false--minting of
<code>currency</code> based upon
<code>allow_minting</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(account: &signer, allow_minting: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(account: &signer, allow_minting: bool) {
    <b>let</b> tc_capability = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;TreasuryComplianceRole&gt;(account);
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_minting_ability">Libra::update_minting_ability</a>&lt;Currency&gt;(&tc_capability, allow_minting);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, tc_capability);
}
</code></pre>



</details>
