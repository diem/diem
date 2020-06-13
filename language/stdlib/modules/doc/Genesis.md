
<a name="0x1_Genesis"></a>

# Module `0x1::Genesis`

### Table of Contents

-  [Function `initialize`](#0x1_Genesis_initialize)



<a name="0x1_Genesis_initialize"></a>

## Function `initialize`



<pre><code><b>fun</b> <a href="#0x1_Genesis_initialize">initialize</a>(association: &signer, tc_account: &signer, tc_addr: address, genesis_auth_key: vector&lt;u8&gt;, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Genesis_initialize">initialize</a>(
    association: &signer,
    tc_account: &signer,
    tc_addr: address,
    genesis_auth_key: vector&lt;u8&gt;,
    publishing_option: vector&lt;u8&gt;,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
) {
    <b>let</b> dummy_auth_key_prefix = x"00000000000000000000000000000000";

    <a href="Roles.md#0x1_Roles_grant_root_association_role">Roles::grant_root_association_role</a>(association);
    <a href="LibraConfig.md#0x1_LibraConfig_grant_privileges">LibraConfig::grant_privileges</a>(association);
    <a href="LibraAccount.md#0x1_LibraAccount_grant_association_privileges">LibraAccount::grant_association_privileges</a>(association);
    <a href="SlidingNonce.md#0x1_SlidingNonce_grant_privileges">SlidingNonce::grant_privileges</a>(association);
    <b>let</b> assoc_root_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;AssociationRootRole&gt;(association);
    <b>let</b> create_config_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;CreateOnChainConfig&gt;(association);
    <b>let</b> create_sliding_nonce_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;CreateSlidingNonce&gt;(association);

    <a href="Roles.md#0x1_Roles_grant_treasury_compliance_role">Roles::grant_treasury_compliance_role</a>(tc_account, &assoc_root_capability);
    <a href="LibraAccount.md#0x1_LibraAccount_grant_treasury_compliance_privileges">LibraAccount::grant_treasury_compliance_privileges</a>(tc_account);
    <a href="Libra.md#0x1_Libra_grant_privileges">Libra::grant_privileges</a>(tc_account);
    <a href="DualAttestationLimit.md#0x1_DualAttestationLimit_grant_privileges">DualAttestationLimit::grant_privileges</a>(tc_account);
    <b>let</b> currency_registration_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;RegisterNewCurrency&gt;(tc_account);
    <b>let</b> tc_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;TreasuryComplianceRole&gt;(tc_account);

    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(association);

    // <a href="Event.md#0x1_Event">Event</a> and On-chain config setup
    <a href="LibraConfig.md#0x1_LibraConfig_initialize">LibraConfig::initialize</a>(
        association,
        &create_config_capability,
    );

    // Currency setup
    <a href="Libra.md#0x1_Libra_initialize">Libra::initialize</a>(association, &create_config_capability);

    // Set that this is testnet
    <a href="Testnet.md#0x1_Testnet_initialize">Testnet::initialize</a>(association);

    // Currency setup
    <b>let</b> (coin1_mint_cap, coin1_burn_cap) = <a href="Coin1.md#0x1_Coin1_initialize">Coin1::initialize</a>(
        association,
        &currency_registration_capability,
    );
    <b>let</b> (coin2_mint_cap, coin2_burn_cap) = <a href="Coin2.md#0x1_Coin2_initialize">Coin2::initialize</a>(
        association,
        &currency_registration_capability,
    );
    <a href="LBR.md#0x1_LBR_initialize">LBR::initialize</a>(
        association,
        &currency_registration_capability,
        &tc_capability,
    );

    <a href="LibraAccount.md#0x1_LibraAccount_initialize">LibraAccount::initialize</a>(association, &assoc_root_capability);
    <a href="LibraAccount.md#0x1_LibraAccount_create_root_association_account">LibraAccount::create_root_association_account</a>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association),
        <b>copy</b> dummy_auth_key_prefix,
    );

    // Register transaction fee <b>resource</b>
    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(
        association,
        &tc_capability,
    );

    // Create the treasury compliance account
    <a href="LibraAccount.md#0x1_LibraAccount_create_treasury_compliance_account">LibraAccount::create_treasury_compliance_account</a>(
        &assoc_root_capability,
        &tc_capability,
        &create_sliding_nonce_capability,
        tc_addr,
        <b>copy</b> dummy_auth_key_prefix,
        coin1_mint_cap,
        coin1_burn_cap,
        coin2_mint_cap,
        coin2_burn_cap,
    );
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>(tc_account);
    <a href="AccountLimits.md#0x1_AccountLimits_certify_limits_definition">AccountLimits::certify_limits_definition</a>(&tc_capability, tc_addr);

    <a href="LibraTransactionTimeout.md#0x1_LibraTransactionTimeout_initialize">LibraTransactionTimeout::initialize</a>(association);
    <a href="LibraSystem.md#0x1_LibraSystem_initialize_validator_set">LibraSystem::initialize_validator_set</a>(association, &create_config_capability);
    <a href="LibraVersion.md#0x1_LibraVersion_initialize">LibraVersion::initialize</a>(association, &create_config_capability);

    <a href="DualAttestationLimit.md#0x1_DualAttestationLimit_initialize">DualAttestationLimit::initialize</a>(association, tc_account, &create_config_capability);
    <a href="LibraBlock.md#0x1_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>(association);
    <a href="LibraWriteSetManager.md#0x1_LibraWriteSetManager_initialize">LibraWriteSetManager::initialize</a>(association);
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_initialize">LibraTimestamp::initialize</a>(association);

    <b>let</b> assoc_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(association);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&assoc_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(assoc_rotate_key_cap);

    <a href="LibraVMConfig.md#0x1_LibraVMConfig_initialize">LibraVMConfig::initialize</a>(
        association,
        association,
        &create_config_capability,
        publishing_option,
        instruction_schedule,
        native_schedule,
    );

    <b>let</b> config_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(association);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&config_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(config_rotate_key_cap);

    <b>let</b> tc_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(tc_account);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&tc_rotate_key_cap, <b>copy</b> genesis_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(tc_rotate_key_cap);

    // Restore privileges
    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(association, create_config_capability);
    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(association, create_sliding_nonce_capability);
    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(association, assoc_root_capability);

    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, currency_registration_capability);
    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, tc_capability);

    // Mark that genesis has finished. This must appear <b>as</b> the last call.
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_set_time_has_started">LibraTimestamp::set_time_has_started</a>(association);
}
</code></pre>



</details>
