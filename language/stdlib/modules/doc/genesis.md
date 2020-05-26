
<a name="0x0_Genesis"></a>

# Module `0x0::Genesis`

### Table of Contents

-  [Function `initialize_association`](#0x0_Genesis_initialize_association)
-  [Function `initialize_accounts`](#0x0_Genesis_initialize_accounts)
-  [Function `initialize_tc_account`](#0x0_Genesis_initialize_tc_account)
-  [Function `grant_tc_account`](#0x0_Genesis_grant_tc_account)
-  [Function `grant_tc_capabilities_for_sender`](#0x0_Genesis_grant_tc_capabilities_for_sender)
-  [Function `initialize_txn_fee_account`](#0x0_Genesis_initialize_txn_fee_account)
-  [Function `initialize_config`](#0x0_Genesis_initialize_config)



<a name="0x0_Genesis_initialize_association"></a>

## Function `initialize_association`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_association">initialize_association</a>(association_root_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_association">initialize_association</a>(association_root_addr: address) {
    // <a href="association.md#0x0_Association">Association</a>/cap setup
    <a href="association.md#0x0_Association_initialize">Association::initialize</a>();
    <a href="association.md#0x0_Association_apply_for_privilege">Association::apply_for_privilege</a>&lt;<a href="libra.md#0x0_Libra_AddCurrency">Libra::AddCurrency</a>&gt;();
    <a href="association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="libra.md#0x0_Libra_AddCurrency">Libra::AddCurrency</a>&gt;(association_root_addr);
}
</code></pre>



</details>

<a name="0x0_Genesis_initialize_accounts"></a>

## Function `initialize_accounts`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_accounts">initialize_accounts</a>(association_root_addr: address, burn_addr: address, genesis_auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_accounts">initialize_accounts</a>(
    association_root_addr: address,
    burn_addr: address,
    genesis_auth_key: vector&lt;u8&gt;,
) {
    <b>let</b> dummy_auth_key = x"00000000000000000000000000000000";

    // Set that this is testnet
    <a href="testnet.md#0x0_Testnet_initialize">Testnet::initialize</a>();

    // <a href="event.md#0x0_Event">Event</a> and currency setup
    <a href="event.md#0x0_Event_grant_event_generator">Event::grant_event_generator</a>();
    <a href="coin1.md#0x0_Coin1_initialize">Coin1::initialize</a>();
    <a href="coin2.md#0x0_Coin2_initialize">Coin2::initialize</a>();
    <a href="lbr.md#0x0_LBR_initialize">LBR::initialize</a>();
    <a href="libra_configs.md#0x0_LibraConfig_apply_for_creator_privilege">LibraConfig::apply_for_creator_privilege</a>();
    <a href="libra_configs.md#0x0_LibraConfig_grant_creator_privilege">LibraConfig::grant_creator_privilege</a>(0xA550C18);

    //// Account type setup
    <a href="account_type.md#0x0_AccountType_register">AccountType::register</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;();
    <a href="account_type.md#0x0_AccountType_register">AccountType::register</a>&lt;<a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;();
    <a href="vasp.md#0x0_VASP_initialize">VASP::initialize</a>();

    <a href="account_tracking.md#0x0_AccountTrack_initialize">AccountTrack::initialize</a>();
    <a href="libra_account.md#0x0_LibraAccount_initialize">LibraAccount::initialize</a>();
    <a href="unhosted.md#0x0_Unhosted_publish_global_limits_definition">Unhosted::publish_global_limits_definition</a>();
    <a href="libra_account.md#0x0_LibraAccount_create_account">LibraAccount::create_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(
        association_root_addr,
        <b>copy</b> dummy_auth_key,
    );

    // Create the burn account
    <a href="libra_account.md#0x0_LibraAccount_create_account">LibraAccount::create_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(burn_addr, <b>copy</b> dummy_auth_key);

    // Register transaction fee accounts
    // TODO: Need <b>to</b> convert this <b>to</b> a different account type than unhosted.
    <a href="libra_account.md#0x0_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(0xFEE, <b>copy</b> dummy_auth_key);

    // Create the config account
    <a href="libra_account.md#0x0_LibraAccount_create_account">LibraAccount::create_account</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(<a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(), dummy_auth_key);

    <a href="libra_transaction_timeout.md#0x0_LibraTransactionTimeout_initialize">LibraTransactionTimeout::initialize</a>();
    <a href="libra_block.md#0x0_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>();
    <a href="libra_writeset_manager.md#0x0_LibraWriteSetManager_initialize">LibraWriteSetManager::initialize</a>();
    <a href="libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(genesis_auth_key);
}
</code></pre>



</details>

<a name="0x0_Genesis_initialize_tc_account"></a>

## Function `initialize_tc_account`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_tc_account">initialize_tc_account</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_tc_account">initialize_tc_account</a>() {
    <a href="association.md#0x0_Association_apply_for_association">Association::apply_for_association</a>();
    <a href="association.md#0x0_Association_apply_for_privilege">Association::apply_for_privilege</a>&lt;<a href="libra_account.md#0x0_LibraAccount_FreezingPrivilege">LibraAccount::FreezingPrivilege</a>&gt;();
}
</code></pre>



</details>

<a name="0x0_Genesis_grant_tc_account"></a>

## Function `grant_tc_account`



<pre><code><b>fun</b> <a href="#0x0_Genesis_grant_tc_account">grant_tc_account</a>(tc_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_grant_tc_account">grant_tc_account</a>(tc_addr: address) {
    <a href="association.md#0x0_Association_grant_association_address">Association::grant_association_address</a>(tc_addr);
    <a href="association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="libra_account.md#0x0_LibraAccount_FreezingPrivilege">LibraAccount::FreezingPrivilege</a>&gt;(tc_addr);
}
</code></pre>



</details>

<a name="0x0_Genesis_grant_tc_capabilities_for_sender"></a>

## Function `grant_tc_capabilities_for_sender`



<pre><code><b>fun</b> <a href="#0x0_Genesis_grant_tc_capabilities_for_sender">grant_tc_capabilities_for_sender</a>(auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_grant_tc_capabilities_for_sender">grant_tc_capabilities_for_sender</a>(auth_key: vector&lt;u8&gt;) {
    <a href="libra.md#0x0_Libra_grant_burn_capability_for_sender">Libra::grant_burn_capability_for_sender</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;();
    <a href="libra.md#0x0_Libra_grant_burn_capability_for_sender">Libra::grant_burn_capability_for_sender</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;();
    <a href="libra.md#0x0_Libra_grant_mint_capability_for_sender">Libra::grant_mint_capability_for_sender</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;();
    <a href="libra.md#0x0_Libra_grant_mint_capability_for_sender">Libra::grant_mint_capability_for_sender</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;();
    <a href="libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(auth_key);
}
</code></pre>



</details>

<a name="0x0_Genesis_initialize_txn_fee_account"></a>

## Function `initialize_txn_fee_account`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_txn_fee_account">initialize_txn_fee_account</a>(auth_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_txn_fee_account">initialize_txn_fee_account</a>(auth_key: vector&lt;u8&gt;) {
    <a href="libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;();
    <a href="libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;();
    <a href="transaction_fee.md#0x0_TransactionFee_initialize_transaction_fees">TransactionFee::initialize_transaction_fees</a>();
    <a href="libra_account.md#0x0_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(auth_key);
}
</code></pre>



</details>

<a name="0x0_Genesis_initialize_config"></a>

## Function `initialize_config`



<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_config">initialize_config</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Genesis_initialize_config">initialize_config</a>() {
    <a href="event.md#0x0_Event_grant_event_generator">Event::grant_event_generator</a>();
    <a href="libra_configs.md#0x0_LibraConfig_initialize_configuration">LibraConfig::initialize_configuration</a>();
    <a href="libra_configs.md#0x0_LibraConfig_apply_for_creator_privilege">LibraConfig::apply_for_creator_privilege</a>();
}
</code></pre>



</details>
