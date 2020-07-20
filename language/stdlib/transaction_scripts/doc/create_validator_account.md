
<a name="SCRIPT"></a>

# Script `create_validator_account.move`

### Table of Contents

-  [Function `create_validator_account`](#SCRIPT_create_validator_account)



<a name="SCRIPT_create_validator_account"></a>

## Function `create_validator_account`

Create a validator account at
<code>new_validator_address</code> with
<code>auth_key_prefix</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>(
    creator: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    ) {
    <b>let</b> rotation_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_account_and_extract_key_rotation_cap">LibraAccount::create_validator_account_and_extract_key_rotation_cap</a>(
        creator,
        new_account_address,
        auth_key_prefix
    );
    // <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(rotation_cap);
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability_explicitly">RecoveryAddress::add_rotation_capability_explicitly</a>(creator, rotation_cap);

}
</code></pre>



</details>
