
<a name="0x1_Genesis"></a>

# Module `0x1::Genesis`

The <code><a href="Genesis.md#0x1_Genesis">Genesis</a></code> module defines the Move initialization entry point of the Diem framework
when executing from a fresh state.

> TODO: Currently there are a few additional functions called from Rust during genesis.
> Document which these are and in which order they are called.


-  [Function `initialize`](#0x1_Genesis_initialize)
-  [Function `initialize_internal`](#0x1_Genesis_initialize_internal)
-  [Function `create_initialize_owners_operators`](#0x1_Genesis_create_initialize_owners_operators)


<pre><code><b>use</b> <a href="AccountFreezing.md#0x1_AccountFreezing">0x1::AccountFreezing</a>;
<b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="Diem.md#0x1_Diem">0x1::Diem</a>;
<b>use</b> <a href="DiemAccount.md#0x1_DiemAccount">0x1::DiemAccount</a>;
<b>use</b> <a href="DiemBlock.md#0x1_DiemBlock">0x1::DiemBlock</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">0x1::DiemConsensusConfig</a>;
<b>use</b> <a href="DiemSystem.md#0x1_DiemSystem">0x1::DiemSystem</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">0x1::DiemTransactionPublishingOption</a>;
<b>use</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig">0x1::DiemVMConfig</a>;
<b>use</b> <a href="DualAttestation.md#0x1_DualAttestation">0x1::DualAttestation</a>;
<b>use</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion">0x1::ExperimentalVersion</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="TransactionFee.md#0x1_TransactionFee">0x1::TransactionFee</a>;
<b>use</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">0x1::ValidatorConfig</a>;
<b>use</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">0x1::ValidatorOperatorConfig</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
<b>use</b> <a href="XDX.md#0x1_XDX">0x1::XDX</a>;
<b>use</b> <a href="XUS.md#0x1_XUS">0x1::XUS</a>;
</code></pre>



<a name="0x1_Genesis_initialize"></a>

## Function `initialize`

Initializes the Diem framework.


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(dr_account: signer, tc_account: signer, dr_auth_key: vector&lt;u8&gt;, tc_auth_key: vector&lt;u8&gt;, initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, is_open_module: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, chain_id: u8, initial_diem_version: u64, consensus_config: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(
    dr_account: signer,
    tc_account: signer,
    dr_auth_key: vector&lt;u8&gt;,
    tc_auth_key: vector&lt;u8&gt;,
    initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    is_open_module: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    chain_id: u8,
    initial_diem_version: u64,
    consensus_config: vector&lt;u8&gt;,
) {
    <a href="Genesis.md#0x1_Genesis_initialize_internal">initialize_internal</a>(
        &dr_account,
        &tc_account,
        dr_auth_key,
        tc_auth_key,
        initial_script_allow_list,
        is_open_module,
        instruction_schedule,
        native_schedule,
        chain_id,
        initial_diem_version,
        consensus_config,
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>

For verification of genesis, the goal is to prove that all the invariants which
become active after the end of this function hold. This cannot be achieved with
modular verification as we do in regular continuous testing. Rather, this module must
be verified **together** with the module(s) which provides the invariant.

> TODO: currently verifying this module together with modules providing invariants
> (see above) times out. This can likely be solved by making more of the initialize
> functions called by this function opaque, and prove the according invariants locally to
> each module.

Assume that this is called in genesis state (no timestamp).


<pre><code><b>requires</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_genesis">DiemTimestamp::is_genesis</a>();
</code></pre>



</details>

<a name="0x1_Genesis_initialize_internal"></a>

## Function `initialize_internal`

Initializes the Diem Framework. Internal so it can be used by both genesis code, and for testing purposes


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_internal">initialize_internal</a>(dr_account: &signer, tc_account: &signer, dr_auth_key: vector&lt;u8&gt;, tc_auth_key: vector&lt;u8&gt;, initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, is_open_module: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, chain_id: u8, initial_diem_version: u64, consensus_config: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_internal">initialize_internal</a>(
    dr_account: &signer,
    tc_account: &signer,
    dr_auth_key: vector&lt;u8&gt;,
    tc_auth_key: vector&lt;u8&gt;,
    initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    is_open_module: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    chain_id: u8,
    initial_diem_version: u64,
    consensus_config: vector&lt;u8&gt;,
) {
    <a href="DiemAccount.md#0x1_DiemAccount_initialize">DiemAccount::initialize</a>(dr_account, x"00000000000000000000000000000000");

    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(dr_account, chain_id);

    // On-chain config setup
    <a href="DiemConfig.md#0x1_DiemConfig_initialize">DiemConfig::initialize</a>(dr_account);

    // Consensus config setup
    <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_initialize">DiemConsensusConfig::initialize</a>(dr_account);

    // Currency setup
    <a href="Diem.md#0x1_Diem_initialize">Diem::initialize</a>(dr_account);

    // Currency setup
    <a href="XUS.md#0x1_XUS_initialize">XUS::initialize</a>(dr_account, tc_account);

    <a href="XDX.md#0x1_XDX_initialize">XDX::initialize</a>(dr_account, tc_account);

    <a href="AccountFreezing.md#0x1_AccountFreezing_initialize">AccountFreezing::initialize</a>(dr_account);
    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(tc_account);

    <a href="DiemSystem.md#0x1_DiemSystem_initialize_validator_set">DiemSystem::initialize_validator_set</a>(dr_account);

    <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_initialize">ExperimentalVersion::initialize</a>(dr_account, initial_diem_version);
    <a href="DualAttestation.md#0x1_DualAttestation_initialize">DualAttestation::initialize</a>(dr_account);
    <a href="DiemBlock.md#0x1_DiemBlock_initialize_block_metadata">DiemBlock::initialize_block_metadata</a>(dr_account);

    // Rotate auth keys for DiemRoot and TreasuryCompliance accounts <b>to</b> the given
    // values
    <b>let</b> dr_rotate_key_cap = <a href="DiemAccount.md#0x1_DiemAccount_extract_key_rotation_capability">DiemAccount::extract_key_rotation_capability</a>(dr_account);
    <a href="DiemAccount.md#0x1_DiemAccount_rotate_authentication_key">DiemAccount::rotate_authentication_key</a>(&dr_rotate_key_cap, dr_auth_key);
    <a href="DiemAccount.md#0x1_DiemAccount_restore_key_rotation_capability">DiemAccount::restore_key_rotation_capability</a>(dr_rotate_key_cap);

    <b>let</b> tc_rotate_key_cap = <a href="DiemAccount.md#0x1_DiemAccount_extract_key_rotation_capability">DiemAccount::extract_key_rotation_capability</a>(tc_account);
    <a href="DiemAccount.md#0x1_DiemAccount_rotate_authentication_key">DiemAccount::rotate_authentication_key</a>(&tc_rotate_key_cap, tc_auth_key);
    <a href="DiemAccount.md#0x1_DiemAccount_restore_key_rotation_capability">DiemAccount::restore_key_rotation_capability</a>(tc_rotate_key_cap);

    <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_initialize">DiemTransactionPublishingOption::initialize</a>(
        dr_account,
        initial_script_allow_list,
        is_open_module,
    );

    <a href="DiemVMConfig.md#0x1_DiemVMConfig_initialize">DiemVMConfig::initialize</a>(
        dr_account,
        instruction_schedule,
        native_schedule,
    );

    <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_set">DiemConsensusConfig::set</a>(dr_account, consensus_config);

    // After we have called this function, all invariants which are guarded by
    // `<a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>() ==&gt; ...` will become active and a verification condition.
    // See also discussion at function specification.
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_set_time_has_started">DiemTimestamp::set_time_has_started</a>(dr_account);
}
</code></pre>



</details>

<a name="0x1_Genesis_create_initialize_owners_operators"></a>

## Function `create_initialize_owners_operators`

Sets up the initial validator set for the Diem network.
The validator "owner" accounts, their UTF-8 names, and their authentication
keys are encoded in the <code>owners</code>, <code>owner_names</code>, and <code>owner_auth_key</code> vectors.
Each validator signs consensus messages with the private key corresponding to the Ed25519
public key in <code>consensus_pubkeys</code>.
Each validator owner has its operation delegated to an "operator" (which may be
the owner). The operators, their names, and their authentication keys are encoded
in the <code>operators</code>, <code>operator_names</code>, and <code>operator_auth_keys</code> vectors.
Finally, each validator must specify the network address
(see diem/types/src/network_address/mod.rs) for itself and its full nodes.


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_create_initialize_owners_operators">create_initialize_owners_operators</a>(dr_account: signer, owners: vector&lt;signer&gt;, owner_names: vector&lt;vector&lt;u8&gt;&gt;, owner_auth_keys: vector&lt;vector&lt;u8&gt;&gt;, consensus_pubkeys: vector&lt;vector&lt;u8&gt;&gt;, operators: vector&lt;signer&gt;, operator_names: vector&lt;vector&lt;u8&gt;&gt;, operator_auth_keys: vector&lt;vector&lt;u8&gt;&gt;, validator_network_addresses: vector&lt;vector&lt;u8&gt;&gt;, full_node_network_addresses: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_create_initialize_owners_operators">create_initialize_owners_operators</a>(
    dr_account: signer,
    owners: vector&lt;signer&gt;,
    owner_names: vector&lt;vector&lt;u8&gt;&gt;,
    owner_auth_keys: vector&lt;vector&lt;u8&gt;&gt;,
    consensus_pubkeys: vector&lt;vector&lt;u8&gt;&gt;,
    operators: vector&lt;signer&gt;,
    operator_names: vector&lt;vector&lt;u8&gt;&gt;,
    operator_auth_keys: vector&lt;vector&lt;u8&gt;&gt;,
    validator_network_addresses: vector&lt;vector&lt;u8&gt;&gt;,
    full_node_network_addresses: vector&lt;vector&lt;u8&gt;&gt;,
) {
    <b>let</b> num_owners = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&owners);
    <b>let</b> num_owner_names = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&owner_names);
    <b>assert</b>!(num_owners == num_owner_names, 0);
    <b>let</b> num_owner_keys = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&owner_auth_keys);
    <b>assert</b>!(num_owner_names == num_owner_keys, 0);
    <b>let</b> num_operators = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&operators);
    <b>assert</b>!(num_owner_keys == num_operators, 0);
    <b>let</b> num_operator_names = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&operator_names);
    <b>assert</b>!(num_operators == num_operator_names, 0);
    <b>let</b> num_operator_keys = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&operator_auth_keys);
    <b>assert</b>!(num_operator_names == num_operator_keys, 0);
    <b>let</b> num_validator_network_addresses = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&validator_network_addresses);
    <b>assert</b>!(num_operator_keys == num_validator_network_addresses, 0);
    <b>let</b> num_full_node_network_addresses = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&full_node_network_addresses);
    <b>assert</b>!(num_validator_network_addresses == num_full_node_network_addresses, 0);

    <b>let</b> i = 0;
    <b>let</b> dummy_auth_key_prefix = x"00000000000000000000000000000000";
    <b>while</b> (i &lt; num_owners) {
        <b>let</b> owner = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&owners, i);
        <b>let</b> owner_address = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(owner);
        <b>let</b> owner_name = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&owner_names, i);
        // create each validator account and rotate its auth key <b>to</b> the correct value
        <a href="DiemAccount.md#0x1_DiemAccount_create_validator_account">DiemAccount::create_validator_account</a>(
            &dr_account, owner_address, <b>copy</b> dummy_auth_key_prefix, owner_name
        );

        <b>let</b> owner_auth_key = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&owner_auth_keys, i);
        <b>let</b> rotation_cap = <a href="DiemAccount.md#0x1_DiemAccount_extract_key_rotation_capability">DiemAccount::extract_key_rotation_capability</a>(owner);
        <a href="DiemAccount.md#0x1_DiemAccount_rotate_authentication_key">DiemAccount::rotate_authentication_key</a>(&rotation_cap, owner_auth_key);
        <a href="DiemAccount.md#0x1_DiemAccount_restore_key_rotation_capability">DiemAccount::restore_key_rotation_capability</a>(rotation_cap);

        <b>let</b> operator = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&operators, i);
        <b>let</b> operator_address = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(operator);
        <b>let</b> operator_name = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&operator_names, i);
        // create the operator account + rotate its auth key <b>if</b> it does not already exist
        <b>if</b> (!<a href="DiemAccount.md#0x1_DiemAccount_exists_at">DiemAccount::exists_at</a>(operator_address)) {
            <a href="DiemAccount.md#0x1_DiemAccount_create_validator_operator_account">DiemAccount::create_validator_operator_account</a>(
                &dr_account, operator_address, <b>copy</b> dummy_auth_key_prefix, <b>copy</b> operator_name
            );
            <b>let</b> operator_auth_key = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&operator_auth_keys, i);
            <b>let</b> rotation_cap = <a href="DiemAccount.md#0x1_DiemAccount_extract_key_rotation_capability">DiemAccount::extract_key_rotation_capability</a>(operator);
            <a href="DiemAccount.md#0x1_DiemAccount_rotate_authentication_key">DiemAccount::rotate_authentication_key</a>(&rotation_cap, operator_auth_key);
            <a href="DiemAccount.md#0x1_DiemAccount_restore_key_rotation_capability">DiemAccount::restore_key_rotation_capability</a>(rotation_cap);
        };
        // assign the operator <b>to</b> its validator
        <b>assert</b>!(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">ValidatorOperatorConfig::get_human_name</a>(operator_address) == operator_name, 0);
        <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">ValidatorConfig::set_operator</a>(owner, operator_address);

        // <b>use</b> the operator account set up the validator config
        <b>let</b> validator_network_address = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&validator_network_addresses, i);
        <b>let</b> full_node_network_address = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&full_node_network_addresses, i);
        <b>let</b> consensus_pubkey = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&consensus_pubkeys, i);
        <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">ValidatorConfig::set_config</a>(
            operator,
            owner_address,
            consensus_pubkey,
            validator_network_address,
            full_node_network_address
        );

        // finally, add this validator <b>to</b> the validator set
        <a href="DiemSystem.md#0x1_DiemSystem_add_validator">DiemSystem::add_validator</a>(&dr_account, owner_address);

        i = i + 1;
    }
}
</code></pre>



</details>
