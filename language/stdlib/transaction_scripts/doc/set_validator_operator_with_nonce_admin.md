
<a name="SCRIPT"></a>

# Script `set_validator_operator_with_nonce_admin.move`

### Table of Contents

-  [Function `set_validator_operator_with_nonce_admin`](#SCRIPT_set_validator_operator_with_nonce_admin)



<a name="SCRIPT_set_validator_operator_with_nonce_admin"></a>

## Function `set_validator_operator_with_nonce_admin`

Set validator operator as 'operator_account' of validator owner 'account' (via Admin Script).
<code>operator_name</code> should match expected from operator account. This script also
takes
<code>sliding_nonce</code>, as a unique nonce for this operation. See
<code>Sliding_nonce.<b>move</b></code> for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_operator_with_nonce_admin">set_validator_operator_with_nonce_admin</a>(
    lr_account: &signer, account: &signer, sliding_nonce: u64, operator_name: vector&lt;u8&gt;, operator_account: address
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>assert</b>(<a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(account, operator_account);
}
</code></pre>



</details>
