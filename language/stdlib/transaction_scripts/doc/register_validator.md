
<a name="SCRIPT"></a>

# Script `register_validator.move`

### Table of Contents

-  [Function `register_validator`](#SCRIPT_register_validator)



<a name="SCRIPT_register_validator"></a>

## Function `register_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_register_validator">register_validator</a>(account: &signer, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, fullnodes_network_identity_pubkey: vector&lt;u8&gt;, fullnodes_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_register_validator">register_validator</a>(
    account: &signer,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    fullnodes_network_identity_pubkey: vector&lt;u8&gt;,
    fullnodes_network_address: vector&lt;u8&gt;,
) {
    <b>let</b> sender = <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
        account,
        sender,
        consensus_pubkey,
        validator_network_identity_pubkey,
        validator_network_address,
        fullnodes_network_identity_pubkey,
        fullnodes_network_address
    );
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(account, sender)
}
</code></pre>



</details>
