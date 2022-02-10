
<a name="0x1_DiemSystem"></a>

# Module `0x1::DiemSystem`

Maintains information about the set of validators used during consensus.
Provides functions to add, remove, and update validators in the
validator set.

> Note: When trying to understand this code, it's important to know that "config"
and "configuration" are used for several distinct concepts.


-  [Resource `ValidatorSetChainMarker`](#0x1_DiemSystem_ValidatorSetChainMarker)
-  [Struct `ValidatorInfo`](#0x1_DiemSystem_ValidatorInfo)
-  [Resource `DiemSystem`](#0x1_DiemSystem_DiemSystem)
-  [Constants](#@Constants_0)
-  [Function `initialize_validator_set`](#0x1_DiemSystem_initialize_validator_set)
-  [Function `set_diem_system_config`](#0x1_DiemSystem_set_diem_system_config)
-  [Function `add_validator`](#0x1_DiemSystem_add_validator)
-  [Function `remove_validator`](#0x1_DiemSystem_remove_validator)
-  [Function `update_config_and_reconfigure`](#0x1_DiemSystem_update_config_and_reconfigure)
-  [Function `get_diem_system_config`](#0x1_DiemSystem_get_diem_system_config)
-  [Function `is_validator`](#0x1_DiemSystem_is_validator)
-  [Function `get_validator_config`](#0x1_DiemSystem_get_validator_config)
-  [Function `validator_set_size`](#0x1_DiemSystem_validator_set_size)
-  [Function `get_ith_validator_address`](#0x1_DiemSystem_get_ith_validator_address)
-  [Function `assert_chain_marker_is_published`](#0x1_DiemSystem_assert_chain_marker_is_published)
-  [Function `get_validator_index_`](#0x1_DiemSystem_get_validator_index_)
-  [Function `update_ith_validator_info_`](#0x1_DiemSystem_update_ith_validator_info_)
-  [Function `is_validator_`](#0x1_DiemSystem_is_validator_)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
<b>use</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">0x1::ValidatorConfig</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemSystem_ValidatorSetChainMarker"></a>

## Resource `ValidatorSetChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="DiemSystem.md#0x1_DiemSystem_ValidatorSetChainMarker">ValidatorSetChainMarker</a>&lt;T&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DiemSystem_ValidatorInfo"></a>

## Struct `ValidatorInfo`

Information about a Validator Owner.


<pre><code><b>struct</b> <a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>
 The address (account) of the Validator Owner
</dd>
<dt>
<code>consensus_voting_power: u64</code>
</dt>
<dd>
 The voting power of the Validator Owner (currently always 1).
</dd>
<dt>
<code>config: <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a></code>
</dt>
<dd>
 Configuration information about the Validator, such as the
 Validator Operator, human name, and info such as consensus key
 and network addresses.
</dd>
<dt>
<code>last_config_update_time: u64</code>
</dt>
<dd>
 The time of last reconfiguration invoked by this validator
 in microseconds
</dd>
</dl>


</details>

<a name="0x1_DiemSystem_DiemSystem"></a>

## Resource `DiemSystem`

The DiemSystem struct stores the validator set and crypto scheme in
DiemConfig. The DiemSystem struct is stored by DiemConfig, which publishes a
DiemConfig<DiemSystem> resource.


<pre><code><b>struct</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> <b>has</b> <b>copy</b>, drop, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>scheme: u8</code>
</dt>
<dd>
 The current consensus crypto scheme.
</dd>
<dt>
<code>validators: vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">DiemSystem::ValidatorInfo</a>&gt;</code>
</dt>
<dd>
 The current validator set.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemSystem_MAX_U64"></a>

The largest possible u64 value


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_MAX_U64">MAX_U64</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_DiemSystem_ECHAIN_MARKER"></a>

The <code><a href="DiemSystem.md#0x1_DiemSystem_ValidatorSetChainMarker">ValidatorSetChainMarker</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 9;
</code></pre>



<a name="0x1_DiemSystem_EINVALID_TRANSACTION_SENDER"></a>

The validator operator is not the operator for the specified validator


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>: u64 = 4;
</code></pre>



<a name="0x1_DiemSystem_EALREADY_A_VALIDATOR"></a>

Tried to add a validator to the validator set that was already in it


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_EALREADY_A_VALIDATOR">EALREADY_A_VALIDATOR</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemSystem_ECONFIG"></a>

The <code><a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_ECONFIG">ECONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemSystem_ECONFIG_UPDATE_RATE_LIMITED"></a>

Rate limited when trying to update config


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_ECONFIG_UPDATE_RATE_LIMITED">ECONFIG_UPDATE_RATE_LIMITED</a>: u64 = 6;
</code></pre>



<a name="0x1_DiemSystem_ECONFIG_UPDATE_TIME_OVERFLOWS"></a>

Validator config update time overflows


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_ECONFIG_UPDATE_TIME_OVERFLOWS">ECONFIG_UPDATE_TIME_OVERFLOWS</a>: u64 = 8;
</code></pre>



<a name="0x1_DiemSystem_EINVALID_PROSPECTIVE_VALIDATOR"></a>

Tried to add a validator with an invalid state to the validator set


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_EINVALID_PROSPECTIVE_VALIDATOR">EINVALID_PROSPECTIVE_VALIDATOR</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemSystem_EMAX_VALIDATORS"></a>

Validator set already at maximum allowed size


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_EMAX_VALIDATORS">EMAX_VALIDATORS</a>: u64 = 7;
</code></pre>



<a name="0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR"></a>

An operation was attempted on an address not in the vaidator set


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>: u64 = 3;
</code></pre>



<a name="0x1_DiemSystem_EVALIDATOR_INDEX"></a>

An out of bounds index for the validator set was encountered


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_EVALIDATOR_INDEX">EVALIDATOR_INDEX</a>: u64 = 5;
</code></pre>



<a name="0x1_DiemSystem_FIVE_MINUTES"></a>

Number of microseconds in 5 minutes


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_FIVE_MINUTES">FIVE_MINUTES</a>: u64 = 300000000;
</code></pre>



<a name="0x1_DiemSystem_MAX_VALIDATORS"></a>

The maximum number of allowed validators in the validator set


<pre><code><b>const</b> <a href="DiemSystem.md#0x1_DiemSystem_MAX_VALIDATORS">MAX_VALIDATORS</a>: u64 = 256;
</code></pre>



<a name="0x1_DiemSystem_initialize_validator_set"></a>

## Function `initialize_validator_set`

Publishes the DiemSystem struct, which contains the current validator set.
Must be invoked by @CoreResources a single time in Genesis.


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_initialize_validator_set">initialize_validator_set</a>&lt;T&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_initialize_validator_set">initialize_validator_set</a>&lt;T&gt;(
    account: &signer,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(!<b>exists</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorSetChainMarker">ValidatorSetChainMarker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECHAIN_MARKER">ECHAIN_MARKER</a>));
    <b>assert</b>!(!<b>exists</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECONFIG">ECONFIG</a>));
    <b>move_to</b>(account, <a href="DiemSystem.md#0x1_DiemSystem_ValidatorSetChainMarker">ValidatorSetChainMarker</a>&lt;T&gt;{});
    <b>move_to</b>(
        account,
        <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
            scheme: 0,
            validators: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>(),
        },
    );
}
</code></pre>



</details>

<a name="0x1_DiemSystem_set_diem_system_config"></a>

## Function `set_diem_system_config`

Copies a DiemSystem struct into the DiemSystem resource
Called by the add, remove, and update functions.


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_set_diem_system_config">set_diem_system_config</a>(value: <a href="DiemSystem.md#0x1_DiemSystem_DiemSystem">DiemSystem::DiemSystem</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_set_diem_system_config">set_diem_system_config</a>(value: <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>) <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECONFIG">ECONFIG</a>)
    );
    // Updates the <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> and <b>emits</b> a reconfigure event.
    <b>let</b> config_ref = <b>borrow_global_mut</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>&gt;(@CoreResources);
    *config_ref = value;
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>

<a name="0x1_DiemSystem_add_validator"></a>

## Function `add_validator`

Adds a new validator to the validator set.


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_add_validator">add_validator</a>&lt;T&gt;(validator_addr: <b>address</b>, _cap: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_add_validator">add_validator</a>&lt;T&gt;(
    validator_addr: <b>address</b>,
    _cap: Cap&lt;T&gt;
) <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <a href="DiemSystem.md#0x1_DiemSystem_assert_chain_marker_is_published">assert_chain_marker_is_published</a>&lt;T&gt;();

    // A prospective validator must have a validator config resource
    <b>assert</b>!(
        <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_addr),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_EINVALID_PROSPECTIVE_VALIDATOR">EINVALID_PROSPECTIVE_VALIDATOR</a>)
    );

    // Bound the validator set size
    <b>assert</b>!(
        <a href="DiemSystem.md#0x1_DiemSystem_validator_set_size">validator_set_size</a>() &lt; <a href="DiemSystem.md#0x1_DiemSystem_MAX_VALIDATORS">MAX_VALIDATORS</a>,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="DiemSystem.md#0x1_DiemSystem_EMAX_VALIDATORS">EMAX_VALIDATORS</a>)
    );

    <b>let</b> diem_system_config = <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>();

    // Ensure that this <b>address</b> is not already a validator
    <b>assert</b>!(
        !<a href="DiemSystem.md#0x1_DiemSystem_is_validator_">is_validator_</a>(validator_addr, &diem_system_config.validators),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_EALREADY_A_VALIDATOR">EALREADY_A_VALIDATOR</a>)
    );

    // it is guaranteed that the config is non-empty
    <b>let</b> config = <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(validator_addr);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> diem_system_config.validators, <a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a> {
        addr: validator_addr,
        config, // <b>copy</b> the config over <b>to</b> ValidatorSet
        consensus_voting_power: 1,
        last_config_update_time: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_microseconds">DiemTimestamp::now_microseconds</a>(),
    });

    <a href="DiemSystem.md#0x1_DiemSystem_set_diem_system_config">set_diem_system_config</a>(diem_system_config);
}
</code></pre>



</details>

<a name="0x1_DiemSystem_remove_validator"></a>

## Function `remove_validator`

Removes a validator, aborts unless called by diem root account


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_remove_validator">remove_validator</a>&lt;T&gt;(validator_addr: <b>address</b>, _cap: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_remove_validator">remove_validator</a>&lt;T&gt;(
    validator_addr: <b>address</b>,
    _cap: Cap&lt;T&gt;
) <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <a href="DiemSystem.md#0x1_DiemSystem_assert_chain_marker_is_published">assert_chain_marker_is_published</a>&lt;T&gt;();

    <b>let</b> diem_system_config = <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>();
    // Ensure that this <b>address</b> is an active validator
    <b>let</b> to_remove_index_vec = <a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(&diem_system_config.validators, validator_addr);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&to_remove_index_vec), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    <b>let</b> to_remove_index = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(&to_remove_index_vec);
    // Remove corresponding <a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a> from the validator set
    _  = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_swap_remove">Vector::swap_remove</a>(&<b>mut</b> diem_system_config.validators, to_remove_index);

    <a href="DiemSystem.md#0x1_DiemSystem_set_diem_system_config">set_diem_system_config</a>(diem_system_config);
}
</code></pre>



</details>

<a name="0x1_DiemSystem_update_config_and_reconfigure"></a>

## Function `update_config_and_reconfigure`

Copy the information from ValidatorConfig into the validator set.
This function makes no changes to the size or the members of the set.
If the config in the ValidatorSet changes, it stores the new DiemSystem
and emits a reconfigurationevent.


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_update_config_and_reconfigure">update_config_and_reconfigure</a>(validator_operator_account: &signer, validator_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_update_config_and_reconfigure">update_config_and_reconfigure</a>(
    validator_operator_account: &signer,
    validator_addr: <b>address</b>,
) <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">ValidatorConfig::get_operator</a>(validator_addr) == <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>)
    );
    <b>let</b> diem_system_config = <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>();
    <b>let</b> to_update_index_vec = <a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(&diem_system_config.validators, validator_addr);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&to_update_index_vec), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    <b>let</b> to_update_index = *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(&to_update_index_vec);
    <b>let</b> is_validator_info_updated = <a href="DiemSystem.md#0x1_DiemSystem_update_ith_validator_info_">update_ith_validator_info_</a>(&<b>mut</b> diem_system_config.validators, to_update_index);
    <b>if</b> (is_validator_info_updated) {
        <b>let</b> validator_info = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> diem_system_config.validators, to_update_index);
        <b>assert</b>!(
            validator_info.last_config_update_time &lt;= <a href="DiemSystem.md#0x1_DiemSystem_MAX_U64">MAX_U64</a> - <a href="DiemSystem.md#0x1_DiemSystem_FIVE_MINUTES">FIVE_MINUTES</a>,
            <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECONFIG_UPDATE_TIME_OVERFLOWS">ECONFIG_UPDATE_TIME_OVERFLOWS</a>)
        );
        <b>assert</b>!(
            <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_microseconds">DiemTimestamp::now_microseconds</a>() &gt; validator_info.last_config_update_time + <a href="DiemSystem.md#0x1_DiemSystem_FIVE_MINUTES">FIVE_MINUTES</a>,
            <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECONFIG_UPDATE_RATE_LIMITED">ECONFIG_UPDATE_RATE_LIMITED</a>)
        );
        validator_info.last_config_update_time = <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_microseconds">DiemTimestamp::now_microseconds</a>();
        <a href="DiemSystem.md#0x1_DiemSystem_set_diem_system_config">set_diem_system_config</a>(diem_system_config);
    }
}
</code></pre>



</details>

<a name="0x1_DiemSystem_get_diem_system_config"></a>

## Function `get_diem_system_config`

Get the DiemSystem configuration from DiemConfig


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>(): <a href="DiemSystem.md#0x1_DiemSystem_DiemSystem">DiemSystem::DiemSystem</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>(): <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    *<b>borrow_global</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>&gt;(@CoreResources)
}
</code></pre>



</details>

<a name="0x1_DiemSystem_is_validator"></a>

## Function `is_validator`

Return true if <code>addr</code> is in the current validator set


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_is_validator">is_validator</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_is_validator">is_validator</a>(addr: <b>address</b>): bool <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="DiemSystem.md#0x1_DiemSystem_is_validator_">is_validator_</a>(addr, &<a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>().validators)
}
</code></pre>



</details>

<a name="0x1_DiemSystem_get_validator_config"></a>

## Function `get_validator_config`

Returns validator config. Aborts if <code>addr</code> is not in the validator set.


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_validator_config">get_validator_config</a>(addr: <b>address</b>): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_validator_config">get_validator_config</a>(addr: <b>address</b>): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a> <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <b>let</b> diem_system_config = <a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>();
    <b>let</b> validator_index_vec = <a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(&diem_system_config.validators, addr);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&validator_index_vec), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    *&(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&diem_system_config.validators, *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(&validator_index_vec))).config
}
</code></pre>



</details>

<a name="0x1_DiemSystem_validator_set_size"></a>

## Function `validator_set_size`

Return the size of the current validator set


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_validator_set_size">validator_set_size</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_validator_set_size">validator_set_size</a>(): u64 <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a> {
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&<a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>().validators)
}
</code></pre>



</details>

<a name="0x1_DiemSystem_get_ith_validator_address"></a>

## Function `get_ith_validator_address`

Get the <code>i</code>'th validator address in the validator set.


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): <b>address</b> <b>acquires</b> <a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>{
    <b>assert</b>!(i &lt; <a href="DiemSystem.md#0x1_DiemSystem_validator_set_size">validator_set_size</a>(), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemSystem.md#0x1_DiemSystem_EVALIDATOR_INDEX">EVALIDATOR_INDEX</a>));
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&<a href="DiemSystem.md#0x1_DiemSystem_get_diem_system_config">get_diem_system_config</a>().validators, i).addr
}
</code></pre>



</details>

<a name="0x1_DiemSystem_assert_chain_marker_is_published"></a>

## Function `assert_chain_marker_is_published`



<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_assert_chain_marker_is_published">assert_chain_marker_is_published</a>&lt;T&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_assert_chain_marker_is_published">assert_chain_marker_is_published</a>&lt;T&gt;() {
    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorSetChainMarker">ValidatorSetChainMarker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemSystem.md#0x1_DiemSystem_ECHAIN_MARKER">ECHAIN_MARKER</a>));
}
</code></pre>



</details>

<a name="0x1_DiemSystem_get_validator_index_"></a>

## Function `get_validator_index_`

Get the index of the validator by address in the <code>validators</code> vector
It has a loop, so there are spec blocks in the code to assert loop invariants.


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">DiemSystem::ValidatorInfo</a>&gt;, addr: <b>address</b>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a>&gt;, addr: <b>address</b>): <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> size = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(validators);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; size) {
        <b>let</b> validator_info_ref = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow">Vector::borrow</a>(validators, i);
        <b>if</b> (validator_info_ref.addr == addr) {
            <b>return</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <b>return</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x1_DiemSystem_update_ith_validator_info_"></a>

## Function `update_ith_validator_info_`

Updates *i*th validator info, if nothing changed, return false.
This function never aborts.


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">DiemSystem::ValidatorInfo</a>&gt;, i: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a>&gt;, i: u64): bool {
    <b>let</b> size = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(validators);
    // This provably cannot happen, but left it here for safety.
    <b>if</b> (i &gt;= size) {
        <b>return</b> <b>false</b>
    };
    <b>let</b> validator_info = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(validators, i);
    // "is_valid" below should always hold based on a <b>global</b> <b>invariant</b> later
    // in the file (which proves <b>if</b> we comment out some other specifications),
    // but it is left here for safety.
    <b>if</b> (!<a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_info.addr)) {
        <b>return</b> <b>false</b>
    };
    <b>let</b> new_validator_config = <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(validator_info.addr);
    // check <b>if</b> information is the same
    <b>let</b> config_ref = &<b>mut</b> validator_info.config;
    <b>if</b> (config_ref == &new_validator_config) {
        <b>return</b> <b>false</b>
    };
    *config_ref = new_validator_config;
    <b>true</b>
}
</code></pre>



</details>

<a name="0x1_DiemSystem_is_validator_"></a>

## Function `is_validator_`

Private function checks for membership of <code>addr</code> in validator set.


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_is_validator_">is_validator_</a>(addr: <b>address</b>, validators_vec_ref: &vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">DiemSystem::ValidatorInfo</a>&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemSystem.md#0x1_DiemSystem_is_validator_">is_validator_</a>(addr: <b>address</b>, validators_vec_ref: &vector&lt;<a href="DiemSystem.md#0x1_DiemSystem_ValidatorInfo">ValidatorInfo</a>&gt;): bool {
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&<a href="DiemSystem.md#0x1_DiemSystem_get_validator_index_">get_validator_index_</a>(validators_vec_ref, addr))
}
</code></pre>



</details>
