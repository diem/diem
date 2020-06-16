
<a name="SCRIPT"></a>

# Script `create_validator_account.move`

### Table of Contents

-  [Function `create_validator_account`](#SCRIPT_create_validator_account)



<a name="SCRIPT_create_validator_account"></a>

## Function `create_validator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>&lt;Token&gt;(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>&lt;Token&gt;(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_account">LibraAccount::create_validator_account</a>&lt;Token&gt;(
        creator,
        new_account_address,
        auth_key_prefix
    );
}
</code></pre>



</details>
