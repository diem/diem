
<a name="SCRIPT"></a>

# Script `update_minting_ability.move`

### Table of Contents

-  [Function `update_minting_ability`](#SCRIPT_update_minting_ability)



<a name="SCRIPT_update_minting_ability"></a>

## Function `update_minting_ability`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(account: &signer, allow_minting: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_minting_ability">update_minting_ability</a>&lt;Currency&gt;(account: &signer, allow_minting: bool) {
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_minting_ability">Libra::update_minting_ability</a>&lt;Currency&gt;(account, allow_minting)
}
</code></pre>



</details>
