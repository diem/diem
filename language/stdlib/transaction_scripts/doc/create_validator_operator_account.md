
<a name="SCRIPT"></a>

# Script `create_validator_operator_account.move`

### Table of Contents

-  [Function `create_validator_operator_account`](#SCRIPT_create_validator_operator_account)



<a name="SCRIPT_create_validator_operator_account"></a>

## Function `create_validator_operator_account`

Create a validator operator account at
<code>new_validator_address</code> with
<code>auth_key_prefix</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_validator_operator_account">create_validator_operator_account</a>(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_validator_operator_account">create_validator_operator_account</a>(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_operator_account">LibraAccount::create_validator_operator_account</a>(
        creator,
        new_account_address,
        auth_key_prefix
    );
}
</code></pre>



</details>
