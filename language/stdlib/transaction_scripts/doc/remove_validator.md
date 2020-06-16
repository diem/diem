
<a name="SCRIPT"></a>

# Script `remove_validator.move`

### Table of Contents

-  [Function `remove_validator`](#SCRIPT_remove_validator)



<a name="SCRIPT_remove_validator"></a>

## Function `remove_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_remove_validator">remove_validator</a>(account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_remove_validator">remove_validator</a>(account: &signer, validator_address: address) {
    <a href="../../modules/doc/LibraSystem.md#0x1_LibraSystem_remove_validator">LibraSystem::remove_validator</a>(account, validator_address);
}
</code></pre>



</details>
