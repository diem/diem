
<a name="SCRIPT"></a>

# Script `set_validator_config.move`

### Table of Contents

-  [Function `set_validator_config`](#SCRIPT_set_validator_config)



<a name="SCRIPT_set_validator_config"></a>

## Function `set_validator_config`

Set validator's config locally.
Does not emit NewEpochEvent, the config is NOT changed in the validator set.
TODO(valerini): rename to register_validator_config to avoid confusion with
set_validator_config_and_reconfigure script.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_config">set_validator_config</a>(account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;vector&lt;u8&gt;&gt;, full_node_network_addresses: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_config">set_validator_config</a>(
    account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;vector&lt;u8&gt;&gt;,
    full_node_network_addresses: vector&lt;vector&lt;u8&gt;&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        full_node_network_addresses
    );
 }
</code></pre>



</details>
