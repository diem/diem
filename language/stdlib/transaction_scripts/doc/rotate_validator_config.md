
<a name="SCRIPT"></a>

# Script `rotate_validator_config.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, fullnodes_network_identity_pubkey: vector&lt;u8&gt;, fullnodes_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(
    account: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    fullnodes_network_identity_pubkey: vector&lt;u8&gt;,
    fullnodes_network_address: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/validator_config.md#0x0_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(account,
                                validator_account,
                                consensus_pubkey,
                                validator_network_identity_pubkey,
                                validator_network_address,
                                fullnodes_network_identity_pubkey,
                                fullnodes_network_address);
    <a href="../../modules/doc/libra_system.md#0x0_LibraSystem_update_and_reconfigure">LibraSystem::update_and_reconfigure</a>();
}
</code></pre>



</details>
