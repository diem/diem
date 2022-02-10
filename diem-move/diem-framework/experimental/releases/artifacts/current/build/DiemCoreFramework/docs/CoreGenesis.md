
<a name="0x1_CoreGenesis"></a>

# Module `0x1::CoreGenesis`



-  [Function `init`](#0x1_CoreGenesis_init)


<pre><code><b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="DiemBlock.md#0x1_DiemBlock">0x1::DiemBlock</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
</code></pre>



<a name="0x1_CoreGenesis_init"></a>

## Function `init`

This can only be called once successfully, since after the first call time will have started.


<pre><code><b>public</b> <b>fun</b> <a href="CoreGenesis.md#0x1_CoreGenesis_init">init</a>(core_resource_account: &signer, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreGenesis.md#0x1_CoreGenesis_init">init</a>(core_resource_account: &signer, chain_id: u8) {
    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(core_resource_account, chain_id);
    <a href="DiemConfig.md#0x1_DiemConfig_initialize">DiemConfig::initialize</a>(core_resource_account);
    <a href="DiemBlock.md#0x1_DiemBlock_initialize_block_metadata">DiemBlock::initialize_block_metadata</a>(core_resource_account);
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_set_time_has_started">DiemTimestamp::set_time_has_started</a>(core_resource_account);
}
</code></pre>



</details>
