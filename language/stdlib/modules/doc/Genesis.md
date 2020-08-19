
<a name="0x1_Genesis"></a>

# Module `0x1::Genesis`

### Table of Contents

-  [Function `initialize`](#0x1_Genesis_initialize)



<a name="0x1_Genesis_initialize"></a>

## Function `initialize`



<pre><code><b>fun</b> <a href="#0x1_Genesis_initialize">initialize</a>(lr_account: &signer, tc_account: &signer, tc_addr: address, genesis_auth_key: vector&lt;u8&gt;, initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, is_open_module: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Genesis_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
    tc_addr: address,
    genesis_auth_key: vector&lt;u8&gt;,
    initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    is_open_module: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    chain_id: u8,
) {
    <b>let</b> dummy_auth_key_prefix = x"00000000000000000000000000000000";

    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(lr_account, chain_id);

    <a href="Roles.md#0x1_Roles_grant_libra_root_role">Roles::grant_libra_root_role</a>(lr_account);
    <a href="Roles.md#0x1_Roles_grant_treasury_compliance_role">Roles::grant_treasury_compliance_role</a>(tc_account, lr_account);

    // <a href="Event.md#0x1_Event">Event</a> and On-chain config setup
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(lr_account);
    <a href="LibraConfig.md#0x1_LibraConfig_initialize">LibraConfig::initialize</a>(lr_account);

    // Currency and <a href="VASP.md#0x1_VASP">VASP</a> setup
    <a href="Libra.md#0x1_Libra_initialize">Libra::initialize</a>(lr_account);
    <a href="VASP.md#0x1_VASP_initialize">VASP::initialize</a>(lr_account);

    // Currency setup
    <a href="Coin1.md#0x1_Coin1_initialize">Coin1::initialize</a>(lr_account, tc_account);
    <a href="Coin2.md#0x1_Coin2_initialize">Coin2::initialize</a>(lr_account, tc_account);

    <a href="LBR.md#0x1_LBR_initialize">LBR::initialize</a>(
        lr_account,
        tc_account,
    );

    <a href="AccountFreezing.md#0x1_AccountFreezing_initialize">AccountFreezing::initialize</a>(lr_account);
    <a href="LibraAccount.md#0x1_LibraAccount_initialize">LibraAccount::initialize</a>(lr_account);
    <a href="LibraAccount.md#0x1_LibraAccount_create_libra_root_account">LibraAccount::create_libra_root_account</a>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account),
        <b>copy</b> dummy_auth_key_prefix,
    );

    // Register transaction fee <b>resource</b>
    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(
        lr_account,
        tc_account,
    );

    // Create the treasury compliance account
    <a href="LibraAccount.md#0x1_LibraAccount_create_treasury_compliance_account">LibraAccount::create_treasury_compliance_account</a>(
        lr_account,
        tc_addr,
        <b>copy</b> dummy_auth_key_prefix,
    );

    <a href="LibraSystem.md#0x1_LibraSystem_initialize_validator_set">LibraSystem::initialize_validator_set</a>(
        lr_account,
    );
    <a href="LibraVersion.md#0x1_LibraVersion_initialize">LibraVersion::initialize</a>(
        lr_account,
    );
    <a href="DualAttestation.md#0x1_DualAttestation_initialize">DualAttestation::initialize</a>(
        lr_account,
    );
    <a href="LibraBlock.md#0x1_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>(lr_account);
    <a href="LibraWriteSetManager.md#0x1_LibraWriteSetManager_initialize">LibraWriteSetManager::initialize</a>(lr_account);
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_initialize">LibraTimestamp::initialize</a>(lr_account);

    <b>let</b> lr_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(lr_account);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&lr_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(lr_rotate_key_cap);

    <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">LibraTransactionPublishingOption::initialize</a>(
        lr_account,
        initial_script_allow_list,
        is_open_module,
    );

    <a href="LibraVMConfig.md#0x1_LibraVMConfig_initialize">LibraVMConfig::initialize</a>(
        lr_account,
        instruction_schedule,
        native_schedule,
    );

    <b>let</b> tc_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(tc_account);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&tc_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(tc_rotate_key_cap);

    // Mark that genesis has finished. This must appear <b>as</b> the last call.
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_set_time_has_started">LibraTimestamp::set_time_has_started</a>(lr_account);
}
</code></pre>



</details>
