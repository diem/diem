
<a name="SCRIPT"></a>

# Script `add_validator.move`

### Table of Contents

-  [Function `add_validator`](#SCRIPT_add_validator)



<a name="SCRIPT_add_validator"></a>

## Function `add_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_validator">add_validator</a>(account: &signer, validator_address: address) {
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_add_validator">LibraSystem::add_validator</a>(account, validator_address);
}
</code></pre>



</details>
