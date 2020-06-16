
<a name="SCRIPT"></a>

# Script `rotate_consensus_pubkey.move`

### Table of Contents

-  [Function `rotate_consensus_pubkey`](#SCRIPT_rotate_consensus_pubkey)



<a name="SCRIPT_rotate_consensus_pubkey"></a>

## Function `rotate_consensus_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_consensus_pubkey">rotate_consensus_pubkey</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_consensus_pubkey">rotate_consensus_pubkey</a>(account: &signer, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_set_consensus_pubkey">ValidatorConfig::set_consensus_pubkey</a>(account, <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), new_key);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_update_and_reconfigure">LibraSystem::update_and_reconfigure</a>(account);
}
</code></pre>



</details>
