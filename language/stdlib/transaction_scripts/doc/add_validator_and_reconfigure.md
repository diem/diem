
<a name="SCRIPT"></a>

# Script `add_validator_and_reconfigure.move`

### Table of Contents

-  [Function `add_validator_and_reconfigure`](#SCRIPT_add_validator_and_reconfigure)



<a name="SCRIPT_add_validator_and_reconfigure"></a>

## Function `add_validator_and_reconfigure`

Add
<code>new_validator</code> to the validator set.
Fails if the
<code>new_validator</code> address is already in the validator set
or does not have a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code> resource stored at the address.
Emits a NewEpochEvent.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_validator_and_reconfigure">add_validator_and_reconfigure</a>(lr_account: &signer, sliding_nonce: u64, validator_name: vector&lt;u8&gt;, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_validator_and_reconfigure">add_validator_and_reconfigure</a>(
    lr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector&lt;u8&gt;,
    validator_address: address
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>assert</b>(<a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">ValidatorConfig::get_human_name</a>(validator_address) == validator_name, 0);
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(lr_account, validator_address);
}
</code></pre>



</details>
