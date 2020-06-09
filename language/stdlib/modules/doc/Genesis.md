
<a name="0x0_Genesis"></a>

# Module `0x0::Genesis`

### Table of Contents

-  [Function `initialize`](#0x0_Genesis_initialize)



<a name="0x0_Genesis_initialize"></a>

## Function `initialize`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize">initialize</a>(association: &signer, config_account: &signer, fee_account: &signer, tc_account: &signer, tc_addr: address, genesis_auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize">initialize</a>(
    association: &signer,
    config_account: &signer,
    fee_account: &signer,
    tc_account: &signer,
    tc_addr: address,
    genesis_auth_key: vector&lt;u8&gt;,
) {
    <b>let</b> dummy_auth_key_prefix = x"00000000000000000000000000000000";

    // <a href="Association.md#0x0_Association">Association</a> root setup
    <a href="Association.md#0x0_Association_initialize">Association::initialize</a>(association);
    <a href="Association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;AddCurrency&gt;(association, association);
    <a href="Association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;PublishModule&gt;(association, association);

    // On-chain config setup
    <a href="Event.md#0x0_Event_publish_generator">Event::publish_generator</a>(config_account);
    <a href="LibraConfig.md#0x0_LibraConfig_initialize">LibraConfig::initialize</a>(config_account, association);

    // Currency setup
    <a href="Libra.md#0x0_Libra_initialize">Libra::initialize</a>(config_account);

    // Set that this is testnet
    <a href="Testnet.md#0x0_Testnet_initialize">Testnet::initialize</a>(association);

    // <a href="Event.md#0x0_Event">Event</a> and currency setup
    <a href="Event.md#0x0_Event_publish_generator">Event::publish_generator</a>(association);
    <b>let</b> (coin1_mint_cap, coin1_burn_cap) = <a href="Coin1.md#0x0_Coin1_initialize">Coin1::initialize</a>(association);
    <b>let</b> (coin2_mint_cap, coin2_burn_cap) = <a href="Coin2.md#0x0_Coin2_initialize">Coin2::initialize</a>(association);
    <a href="LBR.md#0x0_LBR_initialize">LBR::initialize</a>(association);

    <a href="LibraAccount.md#0x0_LibraAccount_initialize">LibraAccount::initialize</a>(association);
    <a href="Unhosted.md#0x0_Unhosted_publish_global_limits_definition">Unhosted::publish_global_limits_definition</a>(association);
    <a href="LibraAccount.md#0x0_LibraAccount_create_genesis_account">LibraAccount::create_genesis_account</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;(
        <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(association),
        <b>copy</b> dummy_auth_key_prefix,
    );
    <a href="Libra.md#0x0_Libra_grant_mint_capability_to_association">Libra::grant_mint_capability_to_association</a>&lt;<a href="Coin1.md#0x0_Coin1">Coin1</a>&gt;(association);
    <a href="Libra.md#0x0_Libra_grant_mint_capability_to_association">Libra::grant_mint_capability_to_association</a>&lt;<a href="Coin2.md#0x0_Coin2">Coin2</a>&gt;(association);

    // Register transaction fee accounts
    <a href="TransactionFee.md#0x0_TransactionFee_initialize">TransactionFee::initialize</a>(association, fee_account, <b>copy</b> dummy_auth_key_prefix);

    // Create the treasury compliance account
    <a href="LibraAccount.md#0x0_LibraAccount_create_treasury_compliance_account">LibraAccount::create_treasury_compliance_account</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;(
        association,
        tc_addr,
        <b>copy</b> dummy_auth_key_prefix,
        coin1_mint_cap,
        coin1_burn_cap,
        coin2_mint_cap,
        coin2_burn_cap,
    );

    // Create the config account
    <a href="LibraAccount.md#0x0_LibraAccount_create_genesis_account">LibraAccount::create_genesis_account</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;(
        <a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>(),
        dummy_auth_key_prefix
    );

    <a href="LibraTransactionTimeout.md#0x0_LibraTransactionTimeout_initialize">LibraTransactionTimeout::initialize</a>(association);
    <a href="LibraSystem.md#0x0_LibraSystem_initialize_validator_set">LibraSystem::initialize_validator_set</a>(config_account);
    <a href="LibraVersion.md#0x0_LibraVersion_initialize">LibraVersion::initialize</a>(config_account);

    <a href="LibraBlock.md#0x0_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>(association);
    <a href="LibraWriteSetManager.md#0x0_LibraWriteSetManager_initialize">LibraWriteSetManager::initialize</a>(association);
    <a href="LibraTimestamp.md#0x0_LibraTimestamp_initialize">LibraTimestamp::initialize</a>(association);

    <b>let</b> assoc_rotate_key_cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(association);
    <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&assoc_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x0_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(assoc_rotate_key_cap);

    <b>let</b> config_rotate_key_cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(config_account);
    <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&config_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x0_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(config_rotate_key_cap);

    <b>let</b> fee_rotate_key_cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(fee_account);
    <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&fee_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x0_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(fee_rotate_key_cap);

    <b>let</b> tc_rotate_key_cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(tc_account);
    <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&tc_rotate_key_cap, genesis_auth_key);
    <a href="LibraAccount.md#0x0_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(tc_rotate_key_cap);
}
</code></pre>



</details>
