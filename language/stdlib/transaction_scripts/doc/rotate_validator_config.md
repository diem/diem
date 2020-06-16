
<a name="SCRIPT"></a>

# Script `rotate_validator_config.move`

### Table of Contents

-  [Function `rotate_validator_config`](#SCRIPT_rotate_validator_config)



<a name="SCRIPT_rotate_validator_config"></a>

## Function `rotate_validator_config`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_validator_config">rotate_validator_config</a>(account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, fullnodes_network_identity_pubkey: vector&lt;u8&gt;, fullnodes_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_validator_config">rotate_validator_config</a>(
    account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    fullnodes_network_identity_pubkey: vector&lt;u8&gt;,
    fullnodes_network_address: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        account,
        validator_account,
        consensus_pubkey,
        validator_network_identity_pubkey,
        validator_network_address,
        fullnodes_network_identity_pubkey,
        fullnodes_network_address
    );
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_update_and_reconfigure">LibraSystem::update_and_reconfigure</a>(account);
}
</code></pre>



</details>
