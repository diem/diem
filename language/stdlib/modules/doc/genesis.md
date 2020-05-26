
<a name="0x0_Genesis"></a>

# Module `0x0::Genesis`

### Table of Contents

-  [Function `initialize`](#0x0_Genesis_initialize)
-  [Function `initialize_txn_fee_account`](#0x0_Genesis_initialize_txn_fee_account)



<a name="0x0_Genesis_initialize"></a>

## Function `initialize`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize">initialize</a>(association: &signer, config_account: &signer, tc_addr: address, tc_auth_key_prefix: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize">initialize</a>(
    association: &signer,
    config_account: &signer,
    tc_addr: address,
    tc_auth_key_prefix: vector&lt;u8&gt;,
    genesis_auth_key: vector&lt;u8&gt;
) {
    <b>let</b> dummy_auth_key_prefix = x"00000000000000000000000000000000";

    // <a href="association.md#0x0_Association">Association</a>/cap setup
    <a href="association.md#0x0_Association_initialize">Association::initialize</a>(association);
    <a href="association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="libra.md#0x0_Libra_AddCurrency">Libra::AddCurrency</a>&gt;(association);

    // On-chain config setup
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(config_account);
    <a href="libra_configs.md#0x0_LibraConfig_initialize">LibraConfig::initialize</a>(config_account, association);

    // Currency setup
    <a href="libra.md#0x0_Libra_initialize">Libra::initialize</a>(config_account);

    // Set that this is testnet
    <a href="testnet.md#0x0_Testnet_initialize">Testnet::initialize</a>();

    // <a href="event.md#0x0_Event">Event</a> and currency setup
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(association);
    <b>let</b> (coin1_mint_cap, coin1_burn_cap) = <a href="coin1.md#0x0_Coin1_initialize">Coin1::initialize</a>(association);
    <b>let</b> (coin2_mint_cap, coin2_burn_cap) = <a href="coin2.md#0x0_Coin2_initialize">Coin2::initialize</a>(association);
    <a href="lbr.md#0x0_LBR_initialize">LBR::initialize</a>(association);

    <a href="libra_account.md#0x0_LibraAccount_initialize">LibraAccount::initialize</a>(association);
    <a href="unhosted.md#0x0_Unhosted_publish_global_limits_definition">Unhosted::publish_global_limits_definition</a>();
    <a href="libra_account.md#0x0_LibraAccount_create_genesis_account">LibraAccount::create_genesis_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(
        <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(association),
        <b>copy</b> dummy_auth_key_prefix,
    );
    <a href="libra.md#0x0_Libra_grant_mint_capability_to_association">Libra::grant_mint_capability_to_association</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(association);
    <a href="libra.md#0x0_Libra_grant_mint_capability_to_association">Libra::grant_mint_capability_to_association</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(association);

    // Register transaction fee accounts
    <a href="libra_account.md#0x0_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(0xFEE, <b>copy</b> dummy_auth_key_prefix);


    // Create the treasury compliance account
    <a href="libra_account.md#0x0_LibraAccount_create_treasury_compliance_account">LibraAccount::create_treasury_compliance_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(
        tc_addr,
        tc_auth_key_prefix,
        coin1_mint_cap,
        coin1_burn_cap,
        coin2_mint_cap,
        coin2_burn_cap,
    );

    // Create the config account
    <a href="libra_account.md#0x0_LibraAccount_create_genesis_account">LibraAccount::create_genesis_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(
        <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(),
        dummy_auth_key_prefix
    );

    <a href="libra_transaction_timeout.md#0x0_LibraTransactionTimeout_initialize">LibraTransactionTimeout::initialize</a>(association);
    <a href="libra_system.md#0x0_LibraSystem_initialize_validator_set">LibraSystem::initialize_validator_set</a>(config_account);
    <a href="libra_version.md#0x0_LibraVersion_initialize">LibraVersion::initialize</a>(config_account);

    <a href="libra_block.md#0x0_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>(association);
    <a href="libra_writeset_manager.md#0x0_LibraWriteSetManager_initialize">LibraWriteSetManager::initialize</a>(association);
    <a href="libra_time.md#0x0_LibraTimestamp_initialize">LibraTimestamp::initialize</a>(association);
    <a href="libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(<b>copy</b> genesis_auth_key);
}
</code></pre>



</details>

<a name="0x0_Genesis_initialize_txn_fee_account"></a>

## Function `initialize_txn_fee_account`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_txn_fee_account">initialize_txn_fee_account</a>(fee_account: &signer, auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_txn_fee_account">initialize_txn_fee_account</a>(fee_account: &signer, auth_key: vector&lt;u8&gt;) {
    // Create the transaction fee account
    <a href="libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(fee_account);
    <a href="libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(fee_account);
    <a href="transaction_fee.md#0x0_TransactionFee_initialize_transaction_fees">TransactionFee::initialize_transaction_fees</a>(fee_account);
    <a href="libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(auth_key);
}
</code></pre>



</details>
