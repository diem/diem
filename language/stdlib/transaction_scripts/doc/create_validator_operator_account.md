
<a name="SCRIPT"></a>

# Script `create_validator_operator_account.move`

### Table of Contents

-  [Function `create_validator_operator_account`](#SCRIPT_create_validator_operator_account)



<a name="SCRIPT_create_validator_operator_account"></a>

## Function `create_validator_operator_account`

Create a validator operator account at
<code>new_validator_address</code> with
<code>auth_key_prefix</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_validator_operator_account">create_validator_operator_account</a>&lt;Token&gt;(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_validator_operator_account">create_validator_operator_account</a>&lt;Token&gt;(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <b>let</b> assoc_root_role = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;LibraRootRole&gt;(creator);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_operator_account">LibraAccount::create_validator_operator_account</a>(
        creator,
        &assoc_root_role,
        new_account_address,
        auth_key_prefix
    );
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(creator, assoc_root_role);
}
</code></pre>



</details>
