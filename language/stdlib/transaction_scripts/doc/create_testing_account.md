
<a name="SCRIPT"></a>

# Script `create_testing_account.move`

### Table of Contents

-  [Function `create_testing_account`](#SCRIPT_create_testing_account)



<a name="SCRIPT_create_testing_account"></a>

## Function `create_testing_account`

Create an account with the ParentVASP role at
<code>address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code> and a 0 balance of type
<code>currency</code>. If
<code>add_all_currencies</code> is true, 0 balances for all available currencies in the system will
also be added. This can only be invoked by an Association account.
The
<code>human_name</code>,
<code>base_url</code>, and compliance_public_key` fields of the
ParentVASP are filled in with dummy information.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_testing_account">create_testing_account</a>&lt;CoinType&gt;(association: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_testing_account">create_testing_account</a>&lt;CoinType&gt;(
    association: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <b>let</b> assoc_root_capability = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;LibraRootRole&gt;(association);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">LibraAccount::create_parent_vasp_account</a>&lt;CoinType&gt;(
        association,
        &assoc_root_capability,
        new_account_address,
        auth_key_prefix,
        b"testnet",
        b"https://libra.org",
        // A bogus (but valid ed25519) compliance <b>public</b> key
        x"b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
        add_all_currencies,
    );
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(association, assoc_root_capability);
}
</code></pre>



</details>
