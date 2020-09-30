
<a name="0x1_LibraSystem"></a>

# Module `0x1::LibraSystem`



-  [Struct <code><a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a></code>](#0x1_LibraSystem_ValidatorInfo)
-  [Resource <code><a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a></code>](#0x1_LibraSystem_CapabilityHolder)
-  [Struct <code><a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a></code>](#0x1_LibraSystem_LibraSystem)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_ECAPABILITY_HOLDER">ECAPABILITY_HOLDER</a></code>](#0x1_LibraSystem_ECAPABILITY_HOLDER)
    -  [Error codes](#@Error_codes_0)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR">EINVALID_PROSPECTIVE_VALIDATOR</a></code>](#0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_EALREADY_A_VALIDATOR">EALREADY_A_VALIDATOR</a></code>](#0x1_LibraSystem_EALREADY_A_VALIDATOR)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a></code>](#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a></code>](#0x1_LibraSystem_EINVALID_TRANSACTION_SENDER)
-  [Const <code><a href="LibraSystem.md#0x1_LibraSystem_EVALIDATOR_INDEX">EVALIDATOR_INDEX</a></code>](#0x1_LibraSystem_EVALIDATOR_INDEX)
-  [Function <code>initialize_validator_set</code>](#0x1_LibraSystem_initialize_validator_set)
-  [Function <code>set_libra_system_config</code>](#0x1_LibraSystem_set_libra_system_config)
-  [Function <code>add_validator</code>](#0x1_LibraSystem_add_validator)
-  [Function <code>remove_validator</code>](#0x1_LibraSystem_remove_validator)
-  [Function <code>update_config_and_reconfigure</code>](#0x1_LibraSystem_update_config_and_reconfigure)
-  [Function <code>get_libra_system_config</code>](#0x1_LibraSystem_get_libra_system_config)
-  [Function <code>is_validator</code>](#0x1_LibraSystem_is_validator)
-  [Function <code>get_validator_config</code>](#0x1_LibraSystem_get_validator_config)
-  [Function <code>validator_set_size</code>](#0x1_LibraSystem_validator_set_size)
-  [Function <code>get_ith_validator_address</code>](#0x1_LibraSystem_get_ith_validator_address)
-  [Function <code>get_validator_index_</code>](#0x1_LibraSystem_get_validator_index_)
-  [Function <code>update_ith_validator_info_</code>](#0x1_LibraSystem_update_ith_validator_info_)
-  [Function <code>is_validator_</code>](#0x1_LibraSystem_is_validator_)
    -  [Module specifications](#@Module_specifications_1)


<a name="0x1_LibraSystem_ValidatorInfo"></a>

## Struct `ValidatorInfo`



<pre><code><b>struct</b> <a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: address</code>
</dt>
<dd>

</dd>
<dt>
<code>consensus_voting_power: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>config: <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraSystem_CapabilityHolder"></a>

## Resource `CapabilityHolder`



<pre><code><b>resource</b> <b>struct</b> <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem_LibraSystem">LibraSystem::LibraSystem</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraSystem_LibraSystem"></a>

## Struct `LibraSystem`



<pre><code><b>struct</b> <a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>
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
<code>validators: vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;</code>
</dt>
<dd>
 The current validator set. Updated only at epoch boundaries via reconfiguration.
</dd>
</dl>


</details>

<details>
<summary>Specification</summary>


Members of <code>validators</code> vector (the validator set) have unique addresses.


<pre><code><b>invariant</b>
    <b>forall</b> i in 0..len(validators), j in 0..len(validators):
        validators[i].addr == validators[j].addr ==&gt; i == j;
</code></pre>



</details>

<a name="0x1_LibraSystem_ECAPABILITY_HOLDER"></a>

## Const `ECAPABILITY_HOLDER`


<a name="@Error_codes_0"></a>

### Error codes

The <code><a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_ECAPABILITY_HOLDER">ECAPABILITY_HOLDER</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR"></a>

## Const `EINVALID_PROSPECTIVE_VALIDATOR`

Tried to add a validator with an invalid state to the validator set


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR">EINVALID_PROSPECTIVE_VALIDATOR</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraSystem_EALREADY_A_VALIDATOR"></a>

## Const `EALREADY_A_VALIDATOR`

Tried to add an existing validator to the validator set


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_EALREADY_A_VALIDATOR">EALREADY_A_VALIDATOR</a>: u64 = 2;
</code></pre>



<a name="0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR"></a>

## Const `ENOT_AN_ACTIVE_VALIDATOR`

An operation was attempted on a non-active validator


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>: u64 = 3;
</code></pre>



<a name="0x1_LibraSystem_EINVALID_TRANSACTION_SENDER"></a>

## Const `EINVALID_TRANSACTION_SENDER`

The validator operator is not the operator for the specified validator


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>: u64 = 4;
</code></pre>



<a name="0x1_LibraSystem_EVALIDATOR_INDEX"></a>

## Const `EVALIDATOR_INDEX`

An out of bounds index for the validator set was encountered


<pre><code><b>const</b> <a href="LibraSystem.md#0x1_LibraSystem_EVALIDATOR_INDEX">EVALIDATOR_INDEX</a>: u64 = 5;
</code></pre>



<a name="0x1_LibraSystem_initialize_validator_set"></a>

## Function `initialize_validator_set`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_initialize_validator_set">initialize_validator_set</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_initialize_validator_set">initialize_validator_set</a>(
    lr_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> cap = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_and_get_capability">LibraConfig::publish_new_config_and_get_capability</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;(
        lr_account,
        <a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a> {
            scheme: 0,
            validators: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
        },
    );
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraSystem.md#0x1_LibraSystem_ECAPABILITY_HOLDER">ECAPABILITY_HOLDER</a>)
    );
    move_to(lr_account, <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a> { cap })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<a name="0x1_LibraSystem_lr_addr$15"></a>
<b>let</b> lr_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account);
<b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>aborts_if</b> <b>exists</b>&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(lr_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <b>exists</b>&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(lr_addr);
<b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;();
<b>ensures</b> len(<a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>()) == 0;
</code></pre>



</details>

<a name="0x1_LibraSystem_set_libra_system_config"></a>

## Function `set_libra_system_config`



<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_set_libra_system_config">set_libra_system_config</a>(value: <a href="LibraSystem.md#0x1_LibraSystem_LibraSystem">LibraSystem::LibraSystem</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_set_libra_system_config">set_libra_system_config</a>(value: <a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>) <b>acquires</b> <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <b>assert</b>(
        <b>exists</b>&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraSystem.md#0x1_LibraSystem_ECAPABILITY_HOLDER">ECAPABILITY_HOLDER</a>)
    );
    <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability_and_reconfigure">LibraConfig::set_with_capability_and_reconfigure</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;(
        &borrow_global&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).cap,
        value
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">LibraConfig::ReconfigureAbortsIf</a>;
<b>ensures</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).payload == value;
</code></pre>



</details>

<a name="0x1_LibraSystem_add_validator"></a>

## Function `add_validator`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_add_validator">add_validator</a>(lr_account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_add_validator">add_validator</a>(
    lr_account: &signer,
    validator_address: address
) <b>acquires</b> <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    // A prospective validator must have a validator config <b>resource</b>
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_address), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_EINVALID_PROSPECTIVE_VALIDATOR">EINVALID_PROSPECTIVE_VALIDATOR</a>));

    <b>let</b> libra_system_config = <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>();
    // Ensure that this address is not already a validator
    <b>assert</b>(
        !<a href="LibraSystem.md#0x1_LibraSystem_is_validator_">is_validator_</a>(validator_address, &libra_system_config.validators),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_EALREADY_A_VALIDATOR">EALREADY_A_VALIDATOR</a>)
    );
    // it is guaranteed that the config is non-empty
    <b>let</b> config = <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(validator_address);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> libra_system_config.validators, <a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a> {
        addr: validator_address,
        config, // <b>copy</b> the config over <b>to</b> ValidatorSet
        consensus_voting_power: 1,
    });

    <a href="LibraSystem.md#0x1_LibraSystem_set_libra_system_config">set_libra_system_config</a>(libra_system_config);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">LibraConfig::ReconfigureAbortsIf</a>;
<b>aborts_if</b> !<a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> <a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(validator_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>


LIP-6 property: validator has validator role. The code does not check this explicitly,
but it is implied by the assert ValidatorConfig::is_valid, since
a published ValidatorConfig has a ValidatorRole is an invariant (in ValidatorConfig).


<pre><code><b>ensures</b> <a href="Roles.md#0x1_Roles_spec_has_validator_role_addr">Roles::spec_has_validator_role_addr</a>(validator_address);
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_address);
<b>ensures</b> <a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(validator_address);
</code></pre>




<a name="0x1_LibraSystem_vs$16"></a>


<pre><code><b>let</b> vs = <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>();
<b>ensures</b> <a href="Vector.md#0x1_Vector_eq_push_back">Vector::eq_push_back</a>(vs,
                             <b>old</b>(vs),
                             <a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a> {
                                 addr: validator_address,
                                 config: <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">ValidatorConfig::spec_get_config</a>(validator_address),
                                 consensus_voting_power: 1,
                              }
                           );
</code></pre>



</details>

<a name="0x1_LibraSystem_remove_validator"></a>

## Function `remove_validator`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_remove_validator">remove_validator</a>(lr_account: &signer, account_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_remove_validator">remove_validator</a>(
    lr_account: &signer,
    account_address: address
) <b>acquires</b> <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> libra_system_config = <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>();
    // Ensure that this address is an active validator
    <b>let</b> to_remove_index_vec = <a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(&libra_system_config.validators, account_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&to_remove_index_vec), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    <b>let</b> to_remove_index = *<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&to_remove_index_vec);
    // Remove corresponding <a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a> from the validator set
    _  = <a href="Vector.md#0x1_Vector_swap_remove">Vector::swap_remove</a>(&<b>mut</b> libra_system_config.validators, to_remove_index);

    <a href="LibraSystem.md#0x1_LibraSystem_set_libra_system_config">set_libra_system_config</a>(libra_system_config);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">LibraConfig::ReconfigureAbortsIf</a>;
<b>aborts_if</b> !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(account_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b> !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(account_address);
</code></pre>




<a name="0x1_LibraSystem_vs$17"></a>


<pre><code><b>let</b> vs = <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>();
<b>ensures</b> <b>forall</b> vi in vs <b>where</b> vi.addr != account_address: <b>exists</b> ovi in <b>old</b>(vs): vi == ovi;
</code></pre>


Removed validator should no longer be valid.


<pre><code><b>ensures</b> !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(account_address);
</code></pre>



</details>

<a name="0x1_LibraSystem_update_config_and_reconfigure"></a>

## Function `update_config_and_reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_update_config_and_reconfigure">update_config_and_reconfigure</a>(validator_operator_account: &signer, validator_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_update_config_and_reconfigure">update_config_and_reconfigure</a>(
    validator_operator_account: &signer,
    validator_address: address,
) <b>acquires</b> <a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_validator_operator">Roles::assert_validator_operator</a>(validator_operator_account);
    <b>assert</b>(
        <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">ValidatorConfig::get_operator</a>(validator_address) == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>)
    );
    <b>let</b> libra_system_config = <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>();
    <b>let</b> to_update_index_vec = <a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(&libra_system_config.validators, validator_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&to_update_index_vec), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    <b>let</b> to_update_index = *<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&to_update_index_vec);
    <b>let</b> is_validator_info_updated = <a href="LibraSystem.md#0x1_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(&<b>mut</b> libra_system_config.validators, to_update_index);
    <b>if</b> (is_validator_info_updated) {
        <a href="LibraSystem.md#0x1_LibraSystem_set_libra_system_config">set_libra_system_config</a>(libra_system_config);
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
</code></pre>


Must abort if the signer does not have the ValidatorOperator role [B23].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidatorOperator">Roles::AbortsIfNotValidatorOperator</a>{validator_operator_account: validator_operator_account};
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">ValidatorConfig::AbortsIfNoValidatorConfig</a>{addr: validator_address};
<b>aborts_if</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">ValidatorConfig::spec_get_operator</a>(validator_address)
    != <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_operator_account)
    <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(validator_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<a name="0x1_LibraSystem_is_validator_info_updated$18"></a>
<b>let</b> is_validator_info_updated =
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validator_address) &&
    (<b>exists</b> v in <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>():
        v.addr == validator_address
        && v.config != <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">ValidatorConfig::spec_get_config</a>(validator_address));
<b>include</b> is_validator_info_updated ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">LibraConfig::ReconfigureAbortsIf</a>;
<b>ensures</b> <a href="Roles.md#0x1_Roles_spec_has_validator_role_addr">Roles::spec_has_validator_role_addr</a>(validator_address);
</code></pre>


*Informally:* Does not change the length of the validator set, only
changes ValidatorInfo for validator_address, and doesn't change
any addresses.

TODO: Look at called procedures to understand this better.  Also,
look at transactions.


<a name="0x1_LibraSystem_vs$19"></a>


<pre><code><b>let</b> vs = <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>();
<b>ensures</b> len(vs) == len(<b>old</b>(vs));
</code></pre>


No addresses change


<pre><code><b>ensures</b> <b>forall</b> i in 0..len(vs): vs[i].addr == <b>old</b>(vs)[i].addr;
</code></pre>


If the validator info address is not the one we're changing, the info does not change.


<pre><code><b>ensures</b> <b>forall</b> i in 0..len(vs) <b>where</b> <b>old</b>(vs)[i].addr != validator_address:
                 vs[i] == <b>old</b>(vs)[i];
</code></pre>


It updates the correct entry in the correct way


<pre><code><b>ensures</b> <b>forall</b> i in 0..len(vs): vs[i].config == <b>old</b>(vs[i].config) ||
            (<b>old</b>(vs)[i].addr == validator_address &&
            vs[i].config == <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(validator_address));
</code></pre>



</details>

<a name="0x1_LibraSystem_get_libra_system_config"></a>

## Function `get_libra_system_config`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>(): <a href="LibraSystem.md#0x1_LibraSystem_LibraSystem">LibraSystem::LibraSystem</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>(): <a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a> {
    <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;()
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;;
<b>ensures</b> result == <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;();
</code></pre>



</details>

<a name="0x1_LibraSystem_is_validator"></a>

## Function `is_validator`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_is_validator">is_validator</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_is_validator">is_validator</a>(addr: address): bool {
    <a href="LibraSystem.md#0x1_LibraSystem_is_validator_">is_validator_</a>(addr, &<a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>().validators)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;;
<b>ensures</b> result == <a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(addr);
</code></pre>




<a name="0x1_LibraSystem_spec_is_validator"></a>


<pre><code><b>define</b> <a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(addr: address): bool {
<b>exists</b> v in <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>(): v.addr == addr
}
</code></pre>



</details>

<a name="0x1_LibraSystem_get_validator_config"></a>

## Function `get_validator_config`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_validator_config">get_validator_config</a>(addr: address): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_validator_config">get_validator_config</a>(addr: address): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a> {
    <b>let</b> libra_system_config = <a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>();
    <b>let</b> validator_index_vec = <a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(&libra_system_config.validators, addr);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&validator_index_vec), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));
    *&(<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&libra_system_config.validators, *<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&validator_index_vec))).config
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;;
<b>aborts_if</b> !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">spec_is_validator</a>(addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b>
    <b>exists</b> info in <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;().validators <b>where</b> info.addr == addr:
        result == info.config;
</code></pre>



</details>

<a name="0x1_LibraSystem_validator_set_size"></a>

## Function `validator_set_size`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_validator_set_size">validator_set_size</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_validator_set_size">validator_set_size</a>(): u64 {
    <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&<a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>().validators)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;;
<b>ensures</b> result == len(<a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>());
</code></pre>



</details>

<a name="0x1_LibraSystem_get_ith_validator_address"></a>

## Function `get_ith_validator_address`



<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): address {
    <b>assert</b>(i &lt; <a href="LibraSystem.md#0x1_LibraSystem_validator_set_size">validator_set_size</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraSystem.md#0x1_LibraSystem_EVALIDATOR_INDEX">EVALIDATOR_INDEX</a>));
    <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&<a href="LibraSystem.md#0x1_LibraSystem_get_libra_system_config">get_libra_system_config</a>().validators, i).addr
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;;
<b>aborts_if</b> i &gt;= len(<a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>()) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b> result == <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>()[i].addr;
</code></pre>



</details>

<a name="0x1_LibraSystem_get_validator_index_"></a>

## Function `get_validator_index_`



<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;, addr: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;, addr: address): <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> size = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(validators);
    <b>let</b> i = 0;
    <b>while</b> ({
        <b>spec</b> {
            <b>assert</b> i &lt;= size;
            <b>assert</b> <b>forall</b> j in 0..i: validators[j].addr != addr;
        };
        (i &lt; size)
    })
    {
        <b>let</b> validator_info_ref = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(validators, i);
        <b>if</b> (validator_info_ref.addr == addr) {
            <b>spec</b> {
                <b>assert</b> validators[i].addr == addr;
            };
            <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <b>spec</b> {
        <b>assert</b> i == size;
        <b>assert</b> <b>forall</b> j in 0..size: validators[j].addr != addr;
    };
    <b>return</b> <a href="Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>aborts_if</b> <b>false</b>;
<a name="0x1_LibraSystem_size$20"></a>
<b>let</b> size = len(validators);
<b>ensures</b> (<b>forall</b> i in 0..size: validators[i].addr != addr) ==&gt; <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(result);
<b>ensures</b>
    (<b>exists</b> i in 0..size: validators[i].addr == addr) ==&gt;
        <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(result) &&
        {
            <b>let</b> at = <a href="Option.md#0x1_Option_spec_get">Option::spec_get</a>(result);
            0 &lt;= at && at &lt; size && validators[at].addr == addr
        };
</code></pre>



</details>

<a name="0x1_LibraSystem_update_ith_validator_info_"></a>

## Function `update_ith_validator_info_`



<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;, i: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;, i: u64): bool {
    <b>let</b> size = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(validators);
    // This provably cannot happen, but leaving it here <b>as</b> a sanity check.
    <b>if</b> (i &gt;= size) {
        <b>return</b> <b>false</b>
    };
    <b>let</b> validator_info = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(validators, i);
    // I believe this cannot happen (depending on an <b>invariant</b> that doesn't terminate right now),
    // but leaving it in <b>as</b> a sanity check.
    // ValidatorConfig::ValidatorConfigRemainsValid
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

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>aborts_if</b> <b>false</b>;
<a name="0x1_LibraSystem_new_validator_config$21"></a>
<b>let</b> new_validator_config = <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">ValidatorConfig::spec_get_config</a>(validators[i].addr);
<b>requires</b> 0 &lt;= i && i &lt; len(validators);
<b>requires</b> [deactivated] <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validators[i].addr);
<b>ensures</b>
    result ==
        (<a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validators[i].addr) &&
         new_validator_config != <b>old</b>(validators[i].config));
<b>ensures</b>
    result ==&gt;
        validators == update_vector(
            <b>old</b>(validators),
            i,
            update_field(<b>old</b>(validators[i]), config, new_validator_config)
        );
<b>ensures</b> !result ==&gt; validators == <b>old</b>(validators);
<b>ensures</b> validators == update_vector(<b>old</b>(validators), i, validators[i]);
</code></pre>



</details>

<a name="0x1_LibraSystem_is_validator_"></a>

## Function `is_validator_`



<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_is_validator_">is_validator_</a>(addr: address, validators_vec_ref: &vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraSystem.md#0x1_LibraSystem_is_validator_">is_validator_</a>(addr: address, validators_vec_ref: &vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&<a href="LibraSystem.md#0x1_LibraSystem_get_validator_index_">get_validator_index_</a>(validators_vec_ref, addr))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == (<b>exists</b> v in validators_vec_ref: v.addr == addr);
</code></pre>



<a name="@Module_specifications_1"></a>

### Module specifications



<a name="0x1_LibraSystem_spec_get_validators"></a>


<pre><code><b>define</b> <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>(): vector&lt;<a href="LibraSystem.md#0x1_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt; {
    <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;().validators
}
</code></pre>


After genesis, the <code><a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a></code> configuration is published, as well as the capability
to modify it. This invariant removes the need to specify aborts_if's for missing
CapabilityHolder at Libra root.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem">LibraSystem</a>&gt;() &&
    <b>exists</b>&lt;<a href="LibraSystem.md#0x1_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The permission "{Add, Remove} Validator" is granted to LibraRoot [B22].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account} <b>to</b> add_validator, remove_validator;
</code></pre>


Restricts the set of functions that can modify the validator set config.
To specify proper access, it is sufficient to show the following conditions,
which are all specified and verified in function specifications, above.
1. <code>initialize</code> aborts if not called during genesis
2. <code>add_validator</code> adds a validator without changing anything else in the validator set
and only completes successfully if the signer is Libra Root.
3. <code>remove_validator</code> removes a validator without changing anything else and only
completes successfully if the signer is Libra Root
4. <code>update_config_and_reconfigure</code> changes only entry for the validator it's supposed
to update, and only completes successfully if the signer is the validator operator
for that validator.
set_libra_system_config is a private function, so it does not have to preserve the property.


<a name="0x1_LibraSystem_ValidatorSetConfigRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraSystem.md#0x1_LibraSystem_ValidatorSetConfigRemainsSame">ValidatorSetConfigRemainsSame</a> {
    <b>ensures</b> <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>() == <b>old</b>(<a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>());
}
</code></pre>



Only {add, remove} validator [B22] and update_config_and_reconfigure
[B23] may change the set of validators in the configuration.


<pre><code><b>apply</b> <a href="LibraSystem.md#0x1_LibraSystem_ValidatorSetConfigRemainsSame">ValidatorSetConfigRemainsSame</a> <b>to</b> *, *&lt;T&gt;
   <b>except</b> add_validator, remove_validator, update_config_and_reconfigure,
       initialize_validator_set, set_libra_system_config;
</code></pre>




<a name="0x1_LibraSystem_validators$22"></a>


<pre><code><b>let</b> validators = <a href="LibraSystem.md#0x1_LibraSystem_spec_get_validators">spec_get_validators</a>();
<b>invariant</b> [deactivated, <b>global</b>] <b>forall</b> i1 in 0..len(validators): <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">ValidatorConfig::is_valid</a>(validators[i1].addr);
</code></pre>



</details>
