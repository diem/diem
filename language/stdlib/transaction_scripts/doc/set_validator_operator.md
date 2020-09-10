
<a name="SCRIPT"></a>

# Script `set_validator_operator.move`

### Table of Contents

-  [Function `set_validator_operator`](#SCRIPT_set_validator_operator)



<a name="SCRIPT_set_validator_operator"></a>

## Function `set_validator_operator`

Set validator's operator


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_operator">set_validator_operator</a>(account: &signer, operator_name: vector&lt;u8&gt;, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_operator">set_validator_operator</a>(
    account: &signer,
    operator_name: vector&lt;u8&gt;,
    operator_account: address
) {
    <b>assert</b>(<a href="../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_account) == operator_name, 0);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(account, operator_account);
}
</code></pre>



</details>
