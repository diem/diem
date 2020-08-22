
<a name="SCRIPT"></a>

# Script `set_validator_config_and_reconfigure.move`

### Table of Contents

-  [Function `set_validator_config_and_reconfigure`](#SCRIPT_set_validator_config_and_reconfigure)



<a name="SCRIPT_set_validator_config_and_reconfigure"></a>

## Function `set_validator_config_and_reconfigure`

Set validator's config and updates the config in the validator set.
NewEpochEvent is emitted.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, fullnodes_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_validator_config_and_reconfigure">set_validator_config_and_reconfigure</a>(
    account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    fullnodes_network_address: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        account,
        validator_account,
        consensus_pubkey,
        validator_network_address,
        fullnodes_network_address
    );
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_update_config_and_reconfigure">LibraSystem::update_config_and_reconfigure</a>(account, validator_account);
 }
</code></pre>



</details>
