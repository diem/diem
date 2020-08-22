
<a name="SCRIPT"></a>

# Script `register_validator_config.move`

### Table of Contents

-  [Function `register_validator_config`](#SCRIPT_register_validator_config)



<a name="SCRIPT_register_validator_config"></a>

## Function `register_validator_config`

Set validator's config locally.
Does not emit NewEpochEvent, the config is NOT changed in the validator set.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_register_validator_config">register_validator_config</a>(account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_register_validator_config">register_validator_config</a>(
    account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses
    );
 }
</code></pre>



</details>
