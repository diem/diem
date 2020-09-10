
<a name="SCRIPT"></a>

# Script `create_validator_account.move`

### Table of Contents

-  [Function `create_validator_account`](#SCRIPT_create_validator_account)



<a name="SCRIPT_create_validator_account"></a>

## Function `create_validator_account`

Create a validator account at
<code>new_validator_address</code> with
<code>auth_key_prefix</code>and human_name.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>(creator: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_validator_account">create_validator_account</a>(
    creator: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    ) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(creator, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_validator_account">LibraAccount::create_validator_account</a>(
        creator,
        new_account_address,
        auth_key_prefix,
        human_name,
    );
}
</code></pre>



</details>
