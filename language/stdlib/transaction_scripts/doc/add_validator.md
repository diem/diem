
<a name="SCRIPT"></a>

# Script `add_validator.move`

### Table of Contents

-  [Function `add_validator`](#SCRIPT_add_validator)



<a name="SCRIPT_add_validator"></a>

## Function `add_validator`

Add
<code>new_validator</code> to the validator set.
Fails if the
<code>new_validator</code> address is already in the validator set
or does not have a
<code><a href="../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code> resource stored at the address.
Emits a NewEpochEvent.
TODO(valerini): rename to add_validator_and_reconfigure?


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(lr_account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(lr_account: &signer, validator_address: address) {
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(lr_account, validator_address);
}
</code></pre>



</details>
