
<a name="0x1_ValidatorConfig"></a>

# Module `0x1::ValidatorConfig`

The ValidatorConfig resource holds information about a validator. Information
is published and updated by Diem root in a <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">Self::ValidatorConfig</a></code> in preparation for
later inclusion (by functions in DiemConfig) in a <code>DiemConfig::DiemConfig&lt;<a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a>&gt;</code>
struct (the <code><a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfig">Self::ValidatorConfig</a></code> in a <code>DiemConfig::ValidatorInfo</code> which is a member
of the <code><a href="DiemSystem.md#0x1_DiemSystem_DiemSystem">DiemSystem::DiemSystem</a>.validators</code> vector).


-  [Resource `ValidatorConfigChainMarker`](#0x1_ValidatorConfig_ValidatorConfigChainMarker)
-  [Struct `Config`](#0x1_ValidatorConfig_Config)
-  [Resource `ValidatorConfig`](#0x1_ValidatorConfig_ValidatorConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_ValidatorConfig_initialize)
-  [Function `publish`](#0x1_ValidatorConfig_publish)
-  [Function `exists_config`](#0x1_ValidatorConfig_exists_config)
-  [Function `set_operator`](#0x1_ValidatorConfig_set_operator)
-  [Function `remove_operator`](#0x1_ValidatorConfig_remove_operator)
-  [Function `set_config`](#0x1_ValidatorConfig_set_config)
-  [Function `is_valid`](#0x1_ValidatorConfig_is_valid)
-  [Function `get_config`](#0x1_ValidatorConfig_get_config)
-  [Function `get_human_name`](#0x1_ValidatorConfig_get_human_name)
-  [Function `get_operator`](#0x1_ValidatorConfig_get_operator)
-  [Function `get_consensus_pubkey`](#0x1_ValidatorConfig_get_consensus_pubkey)
-  [Function `get_validator_network_addresses`](#0x1_ValidatorConfig_get_validator_network_addresses)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signature.md#0x1_Signature">0x1::Signature</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
<b>use</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">0x1::ValidatorOperatorConfig</a>;
</code></pre>



<a name="0x1_ValidatorConfig_ValidatorConfigChainMarker"></a>

## Resource `ValidatorConfigChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfigChainMarker">ValidatorConfigChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_ValidatorConfig_Config"></a>

## Struct `Config`



<pre><code><b>struct</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> <b>has</b> <b>copy</b>, drop, store
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



<pre><code><b>struct</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>&gt;</code>
</dt>
<dd>
 set and rotated by the operator_account
</dd>
<dt>
<code>operator_account: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_Option">Option::Option</a>&lt;<b>address</b>&gt;</code>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ValidatorConfig_ECHAIN_MARKER"></a>

The <code>ValidatorSetChainMarker</code> resource was not in the required state


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 9;
</code></pre>



<a name="0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY"></a>

The provided consensus public key is malformed


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">EINVALID_CONSENSUS_KEY</a>: u64 = 2;
</code></pre>



<a name="0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER"></a>

The sender is not the operator for the specified validator


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>: u64 = 1;
</code></pre>



<a name="0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR"></a>

Tried to set an account without the correct operator role as a Validator Operator


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ENOT_A_VALIDATOR_OPERATOR</a>: u64 = 3;
</code></pre>



<a name="0x1_ValidatorConfig_EVALIDATOR_CONFIG"></a>

The <code><a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_ValidatorConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_initialize">initialize</a>&lt;T&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_initialize">initialize</a>&lt;T&gt;(account: &signer) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfigChainMarker">ValidatorConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );
    <b>move_to</b>(account, <a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfigChainMarker">ValidatorConfigChainMarker</a>&lt;T&gt;{});
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_publish"></a>

## Function `publish`

Publishes a mostly empty ValidatorConfig struct. Eventually, it
will have critical info such as keys, network addresses for validators,
and the address of the validator operator.


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">publish</a>&lt;T&gt;(validator_account: &signer, human_name: vector&lt;u8&gt;, _cap: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">publish</a>&lt;T&gt;(
    validator_account: &signer,
    human_name: vector&lt;u8&gt;,
    _cap: Cap&lt;T&gt;
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig_ValidatorConfigChainMarker">ValidatorConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account)),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>)
    );
    <b>move_to</b>(validator_account, <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
        config: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>(),
        operator_account: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>(),
        human_name,
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_exists_config"></a>

## Function `exists_config`

Returns true if a ValidatorConfig resource exists under addr.


<pre><code><b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_operator"></a>

## Function `set_operator`

Sets a new operator account, preserving the old config.
Note: Access control.  No one but the owner of the account may change .operator_account


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">set_operator</a>(validator_account: &signer, operator_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_operator">set_operator</a>(validator_account: &signer, operator_addr: <b>address</b>) <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>!(
        <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(operator_addr),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_ENOT_A_VALIDATOR_OPERATOR">ENOT_A_VALIDATOR_OPERATOR</a>)
    );
    <b>let</b> sender = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account);
    <b>assert</b>!(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(sender), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    (<b>borrow_global_mut</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(operator_addr);
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
    <b>let</b> sender = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_account);
    // <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> field remains set
    <b>assert</b>!(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(sender), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    (<b>borrow_global_mut</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a>();
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_config"></a>

## Function `set_config`

Rotate the config in the validator_account.
Once the config is set, it can not go back to <code><a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_none">Option::none</a></code> - this is crucial for validity
of the DiemSystem's code.


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">set_config</a>(validator_operator_account: &signer, validator_addr: <b>address</b>, consensus_pubkey: vector&lt;u8&gt;, validator_network_addresses: vector&lt;u8&gt;, fullnode_network_addresses: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_set_config">set_config</a>(
    validator_operator_account: &signer,
    validator_addr: <b>address</b>,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_addresses: vector&lt;u8&gt;,
    fullnode_network_addresses: vector&lt;u8&gt;,
) <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>!(
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account) == <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(validator_addr),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_TRANSACTION_SENDER">EINVALID_TRANSACTION_SENDER</a>)
    );
    <b>assert</b>!(
        <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> consensus_pubkey),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EINVALID_CONSENSUS_KEY">EINVALID_CONSENSUS_KEY</a>)
    );
    // TODO(valerini): verify the proof of posession for consensus_pubkey
    <b>assert</b>!(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(validator_addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = <b>borrow_global_mut</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(validator_addr);
    t_ref.config = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_some">Option::some</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> {
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses,
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_is_valid"></a>

## Function `is_valid`

Returns true if all of the following is true:
1) there is a ValidatorConfig resource under the address, and
2) the config is set, and
we do not require the operator_account to be set to make sure
that if the validator account becomes valid, it stays valid, e.g.
all validators in the Validator Set are valid


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: <b>address</b>): bool <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr) && <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&<b>borrow_global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_config"></a>

## Function `get_config`

Get Config
Aborts if there is no ValidatorConfig resource or if its config is empty


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">get_config</a>(addr: <b>address</b>): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_config">get_config</a>(addr: <b>address</b>): <a href="ValidatorConfig.md#0x1_ValidatorConfig_Config">Config</a> <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>!(<a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">exists_config</a>(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> config = &<b>borrow_global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config;
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(config), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_human_name"></a>

## Function `get_human_name`

Get validator's account human name
Aborts if there is no ValidatorConfig resource


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">get_human_name</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_human_name">get_human_name</a>(addr: <b>address</b>): vector&lt;u8&gt; <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = <b>borrow_global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr);
    *&t_ref.human_name
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_operator"></a>

## Function `get_operator`

Get operator's account
Aborts if there is no ValidatorConfig resource or
if the operator_account is unset


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: <b>address</b>): <b>address</b> <b>acquires</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    <b>let</b> t_ref = <b>borrow_global</b>&lt;<a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr);
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_is_some">Option::is_some</a>(&t_ref.operator_account), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ValidatorConfig.md#0x1_ValidatorConfig_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>));
    *<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option_borrow">Option::borrow</a>(&t_ref.operator_account)
}
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
