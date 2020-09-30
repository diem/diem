
<a name="0x1_ValidatorConfig"></a>

# Module `0x1::ValidatorConfig`



-  [Struct <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a></code>](#0x1_ValidatorConfig_Config)
-  [Resource <code><a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code>](#0x1_ValidatorConfig_ValidatorConfig)
-  [Const <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a></code>](#0x1_ValidatorConfig_EVALIDATOR_CONFIG)
-  [Const <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a></code>](#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER)
-  [Const <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">EINVALID_CONSENSUS_KEY</a></code>](#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY)
-  [Const <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ENOT_A_VALIDATOR_OPERATOR</a></code>](#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR)
-  [Function <code>publish</code>](#0x1_ValidatorConfig_publish)
-  [Function <code>exists_config</code>](#0x1_ValidatorConfig_exists_config)
-  [Function <code>set_operator</code>](#0x1_ValidatorConfig_set_operator)
-  [Function <code>remove_operator</code>](#0x1_ValidatorConfig_remove_operator)
-  [Function <code>set_config</code>](#0x1_ValidatorConfig_set_config)
-  [Function <code>is_valid</code>](#0x1_ValidatorConfig_is_valid)
    -  [Validator stays valid once it becomes valid](#@Validator_stays_valid_once_it_becomes_valid_0)
-  [Function <code>get_config</code>](#0x1_ValidatorConfig_get_config)
-  [Function <code>get_human_name</code>](#0x1_ValidatorConfig_get_human_name)
-  [Function <code>get_operator</code>](#0x1_ValidatorConfig_get_operator)
-  [Function <code>get_consensus_pubkey</code>](#0x1_ValidatorConfig_get_consensus_pubkey)
-  [Function <code>get_validator_network_addresses</code>](#0x1_ValidatorConfig_get_validator_network_addresses)
-  [Module Specification](#@Module_Specification_1)


<a name="0x1_ValidatorConfig_Config"></a>

## Struct `Config`



<pre><code><b>struct</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>consensus_pubkey: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>validator_network_addresses: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>fullnode_network_addresses: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ValidatorConfig_ValidatorConfig"></a>

## Resource `ValidatorConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>&gt;</code>
</dt>
<dd>
 set and rotated by the operator_account
</dd>
<dt>
<code>operator_account: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;address&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>human_name: vector&lt;u8&gt;</code>
</dt>
<dd>
 The human readable name of this entity. Immutable.
</dd>
</dl>


</details>

<a name="0x1_ValidatorConfig_EVALIDATOR_CONFIG"></a>

## Const `EVALIDATOR_CONFIG`

The <code><a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER"></a>

## Const `EINVALID_TRANSACTION_SENDER`

The sender is not the operator for the specified validator


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>: u64 = 1;
</code></pre>



<a name="0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY"></a>

## Const `EINVALID_CONSENSUS_KEY`

The provided consensus public key is malformed


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">EINVALID_CONSENSUS_KEY</a>: u64 = 2;
</code></pre>



<a name="0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR"></a>

## Const `ENOT_A_VALIDATOR_OPERATOR`

Tried to set an account without the correct operator role as a Validator Operator


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ENOT_A_VALIDATOR_OPERATOR</a>: u64 = 3;
</code></pre>



<a name="0x1_ValidatorConfig_publish"></a>

## Function `publish`

Publishes a mostly empty ValidatorConfig struct. Eventually, it
will have critical info such as keys, network addresses for validators,
and the address of the validator operator.


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">publish</a>(validator_account: &signer, lr_account: &signer, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">publish</a>(
    validator_account: &signer,
    lr_account: &signer,
    human_name: vector&lt;u8&gt;,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="Roles.md#0x1_Roles_assert_validator">Roles::assert_validator</a>(validator_account);
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>)
    );
    move_to(validator_account, <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
        config: <a href="Option.md#0x1_Option_none">Option::none</a>(),
        operator_account: <a href="Option.md#0x1_Option_none">Option::none</a>(),
        human_name,
    });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidator">Roles::AbortsIfNotValidator</a>;
<b>aborts_if</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_account))
    <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_account));
</code></pre>



</details>

<a name="0x1_ValidatorConfig_exists_config"></a>

## Function `exists_config`

Returns true if a ValidatorConfig resource exists under addr.


<pre><code><b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_operator"></a>

## Function `set_operator`

Sets a new operator account, preserving the old config.
Note: Access control.  No one but the owner of the account may change .operator_account


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">set_operator</a>(validator_account: &signer, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">set_operator</a>(validator_account: &signer, operator_account: address) <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <a href="Roles.md#0x1_Roles_assert_validator">Roles::assert_validator</a>(validator_account);
    // Check for validator role is not necessary since the role is checked when the config
    // <b>resource</b> is published.
    // TODO (dd): Probably need <b>to</b> prove an <b>invariant</b> about role.
    <b>assert</b>(
        <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(operator_account),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ENOT_A_VALIDATOR_OPERATOR</a>)
    );
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account);
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(sender), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    (borrow_global_mut&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="Option.md#0x1_Option_some">Option::some</a>(operator_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the Validator role [B24].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidator">Roles::AbortsIfNotValidator</a>;
<b>aborts_if</b> !<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(operator_account)
    <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<a name="0x1_ValidatorConfig_sender$16"></a>
<b>let</b> sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_account);
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>{addr: sender};
<b>aborts_if</b> !<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(operator_account) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(sender);
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(sender) == operator_account;
</code></pre>


The signer can only change its own operator account [B24].


<pre><code><b>ensures</b> <b>forall</b> addr: address <b>where</b> addr != sender:
    <b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account == <b>old</b>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account);
</code></pre>



Returns true if addr has an operator account.


<a name="0x1_ValidatorConfig_spec_has_operator"></a>


<pre><code><b>define</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account)
}
</code></pre>


Returns the operator account of a validator if it has one,
and returns the addr itself otherwise.


<a name="0x1_ValidatorConfig_spec_get_operator"></a>


<pre><code><b>define</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(addr: address): address {
    <b>if</b> (<a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(addr)) {
        <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account)
    } <b>else</b> {
        addr
    }
}
</code></pre>


Returns the human name of the validator


<a name="0x1_ValidatorConfig_spec_get_human_name"></a>


<pre><code><b>define</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_human_name">spec_get_human_name</a>(addr: address): vector&lt;u8&gt; {
    <b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).human_name
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_remove_operator"></a>

## Function `remove_operator`

Removes an operator account, setting a corresponding field to Option::none.
The old config is preserved.


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_remove_operator">remove_operator</a>(validator_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_remove_operator">remove_operator</a>(validator_account: &signer) <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <a href="Roles.md#0x1_Roles_assert_validator">Roles::assert_validator</a>(validator_account);
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account);
    // <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> field remains set
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(sender), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    (borrow_global_mut&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="Option.md#0x1_Option_none">Option::none</a>();
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the Validator role [B24].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidator">Roles::AbortsIfNotValidator</a>;
<a name="0x1_ValidatorConfig_sender$17"></a>
<b>let</b> sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_account);
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>{addr: sender};
<b>ensures</b> !<a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_account));
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(sender) == sender;
</code></pre>


The signer can only change its own operator account [B24].


<pre><code><b>ensures</b> <b>forall</b> addr: address <b>where</b> addr != sender:
    <b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account == <b>old</b>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account);
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_config"></a>

## Function `set_config`

Rotate the config in the validator_account
NB! Once the config is set, it can not go to Option::none - this is crucial for validity
of the LibraSystem's code
label: ValidatorConfigRemainsValid


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">set_config</a>(signer: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">set_config</a>(
    signer: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(validator_account),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>)
    );
    <b>assert</b>(
        <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> consensus_pubkey),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">EINVALID_CONSENSUS_KEY</a>)
    );
    // TODO(valerini): verify the proof of posession for consensus_pubkey
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(validator_account), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(validator_account);
    t_ref.config = <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> {
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses,
    });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_ValidatorConfig_sender$18"></a>


<pre><code><b>let</b> sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer);
<b>aborts_if</b> sender != <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(validator_account) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>{addr: validator_account};
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(consensus_pubkey) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(validator_account);
</code></pre>


Returns true if there a config published under addr.


<a name="0x1_ValidatorConfig_spec_has_config"></a>


<pre><code><b>define</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(addr: address): bool {
<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_is_valid"></a>

## Function `is_valid`

Returns true if all of the following is true:
1) there is a ValidatorConfig resource under the address, and
2) the config is set, and
NB! currently we do not require the the operator_account to be set


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr) && <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&borrow_global&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(addr);
</code></pre>



<a name="@Validator_stays_valid_once_it_becomes_valid_0"></a>

### Validator stays valid once it becomes valid

See comment on ValidatorConfig::set_config -- LibraSystem depends on this.
ref: (#ValidatorConfigRemainsValid)


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <b>forall</b> validator: address <b>where</b> <b>old</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(validator)): <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(validator);
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_config"></a>

## Function `get_config`

Get Config
Aborts if there is no ValidatorConfig resource of if its config is empty


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> config = &borrow_global&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config;
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(config), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    *<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(config)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>;
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b> result == <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">spec_get_config</a>(addr);
</code></pre>


Returns the config published under addr.


<a name="0x1_ValidatorConfig_spec_get_config"></a>


<pre><code><b>define</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_config">spec_get_config</a>(addr: address): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> {
<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_human_name"></a>

## Function `get_human_name`

Get validator's account human name
Aborts if there is no ValidatorConfig resource


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = borrow_global&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr);
    *&t_ref.human_name
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>;
<b>ensures</b> result == <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_human_name">spec_get_human_name</a>(addr);
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_operator"></a>

## Function `get_operator`

Get operator's account
Aborts if there is no ValidatorConfig resource, if its operator_account is
empty, returns the input


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: address): address <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = borrow_global&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr);
    *<a href="Option.md#0x1_Option_borrow_with_default">Option::borrow_with_default</a>(&t_ref.operator_account, &addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>include</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_AbortsIfNoValidatorConfig">AbortsIfNoValidatorConfig</a>;
<b>ensures</b> result == <a href="ValidatorConfig.md#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(addr);
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_consensus_pubkey"></a>

## Function `get_consensus_pubkey`

Get consensus_pubkey from Config
Never aborts


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.consensus_pubkey
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_validator_network_addresses"></a>

## Function `get_validator_network_addresses`

Get validator's network address from Config
Never aborts


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_validator_network_addresses">get_validator_network_addresses</a>(config_ref: &<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_validator_network_addresses">get_validator_network_addresses</a>(config_ref: &<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.validator_network_addresses
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<pre><code>pragma aborts_if_is_strict = <b>true</b>;
</code></pre>


Specifies that only set_operator and remove_operator may change the operator for a
particular (validator owner) address. Those two functions have a &signer argument for
the validator account, so we know that the change has been authorized by the validator
owner via signing the transaction. But other functions in this module could also
change the operator_account field of ValidatorConfig, and this shows that they do not.


<a name="0x1_ValidatorConfig_OperatorRemainsSame"></a>


<pre><code><b>schema</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_OperatorRemainsSame">OperatorRemainsSame</a> {
    <b>ensures</b> <b>forall</b> addr1: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr1)):
        <b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr1).operator_account == <b>old</b>(<b>global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr1).operator_account);
}
</code></pre>



set_operator, remove_operator can change the operator account [B24].


<pre><code><b>apply</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_OperatorRemainsSame">OperatorRemainsSame</a> <b>to</b> * <b>except</b> set_operator, remove_operator;
</code></pre>


LIP-6 Property: If address has a ValidatorConfig, it has a validator role.  This invariant is useful
in LibraSystem so we don't have to check whether every validator address has a validator role.
<a name="ValidatorConfigImpliesValidatorRole"></a>


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr1):
    <a href="Roles.md#0x1_Roles_spec_has_validator_role_addr">Roles::spec_has_validator_role_addr</a>(addr1);
</code></pre>
