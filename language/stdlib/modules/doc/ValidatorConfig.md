
<a name="0x1_ValidatorConfig"></a>

# Module `0x1::ValidatorConfig`

### Table of Contents

-  [Resource `UpdateValidatorConfig`](#0x1_ValidatorConfig_UpdateValidatorConfig)
-  [Struct `Config`](#0x1_ValidatorConfig_Config)
-  [Resource `ValidatorConfig`](#0x1_ValidatorConfig_ValidatorConfig)
-  [Function `publish`](#0x1_ValidatorConfig_publish)
-  [Function `set_operator`](#0x1_ValidatorConfig_set_operator)
-  [Function `remove_operator`](#0x1_ValidatorConfig_remove_operator)
-  [Function `set_config`](#0x1_ValidatorConfig_set_config)
-  [Function `is_valid`](#0x1_ValidatorConfig_is_valid)
-  [Function `get_config`](#0x1_ValidatorConfig_get_config)
-  [Function `get_operator`](#0x1_ValidatorConfig_get_operator)
-  [Function `get_consensus_pubkey`](#0x1_ValidatorConfig_get_consensus_pubkey)
-  [Function `get_validator_network_identity_pubkey`](#0x1_ValidatorConfig_get_validator_network_identity_pubkey)
-  [Function `get_validator_network_address`](#0x1_ValidatorConfig_get_validator_network_address)
-  [Specification](#0x1_ValidatorConfig_Specification)
    -  [Function `publish`](#0x1_ValidatorConfig_Specification_publish)
    -  [Function `set_operator`](#0x1_ValidatorConfig_Specification_set_operator)
    -  [Function `remove_operator`](#0x1_ValidatorConfig_Specification_remove_operator)
    -  [Function `set_config`](#0x1_ValidatorConfig_Specification_set_config)
    -  [Function `is_valid`](#0x1_ValidatorConfig_Specification_is_valid)
        -  [Validator stays valid once it becomes valid](#0x1_ValidatorConfig_@Validator_stays_valid_once_it_becomes_valid)
    -  [Function `get_config`](#0x1_ValidatorConfig_Specification_get_config)
    -  [Function `get_operator`](#0x1_ValidatorConfig_Specification_get_operator)



<a name="0x1_ValidatorConfig_UpdateValidatorConfig"></a>

## Resource `UpdateValidatorConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ValidatorConfig_UpdateValidatorConfig">UpdateValidatorConfig</a>
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



<pre><code><b>struct</b> <a href="#0x1_ValidatorConfig_Config">Config</a>
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

<code>validator_network_identity_pubkey: vector&lt;u8&gt;</code>
</dt>
<dd>
 TODO(philiphayes): restructure
   3) remove validator_network_identity_pubkey
   4) remove full_node_network_identity_pubkey
</dd>
<dt>

<code>validator_network_address: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>full_node_network_identity_pubkey: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>full_node_network_address: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ValidatorConfig_ValidatorConfig"></a>

## Resource `ValidatorConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>config: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>&gt;</code>
</dt>
<dd>
 set and rotated by the operator_account
</dd>
<dt>

<code>operator_account: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ValidatorConfig_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_publish">publish</a>(account: &signer, lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_publish">publish</a>(
    account: &signer,
    lr_account: &signer,
    ) {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
    move_to(account, <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
        config: <a href="Option.md#0x1_Option_none">Option::none</a>(),
        operator_account: <a href="Option.md#0x1_Option_none">Option::none</a>(),
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_operator"></a>

## Function `set_operator`

Sets a new operator account, preserving the old config.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_operator">set_operator</a>(account: &signer, operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_operator">set_operator</a>(account: &signer, operator_account: address) <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    (borrow_global_mut&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="Option.md#0x1_Option_some">Option::some</a>(operator_account);
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_remove_operator"></a>

## Function `remove_operator`

Removes an operator account, setting a corresponding field to Option::none.
The old config is preserved.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_remove_operator">remove_operator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_remove_operator">remove_operator</a>(account: &signer) <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // <a href="#0x1_ValidatorConfig_Config">Config</a> field remains set
    (borrow_global_mut&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(sender)).operator_account = <a href="Option.md#0x1_Option_none">Option::none</a>();
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_set_config"></a>

## Function `set_config`

Rotate the config in the validator_account
NB! Once the config is set, it can not go to Option::none - this is crucial for validity
of the LibraSystem's code


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_config">set_config</a>(signer: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, full_node_network_identity_pubkey: vector&lt;u8&gt;, full_node_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_config">set_config</a>(
    signer: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    full_node_network_identity_pubkey: vector&lt;u8&gt;,
    full_node_network_address: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == <a href="#0x1_ValidatorConfig_get_operator">get_operator</a>(validator_account),
        EINVALID_TRANSACTION_SENDER
    );
    <b>assert</b>(<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> consensus_pubkey), EINVALID_CONSENSUS_KEY);
    // TODO(valerini): verify the proof of posession for consensus_pubkey
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(validator_account);
    t_ref.config = <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="#0x1_ValidatorConfig_Config">Config</a> {
        consensus_pubkey,
        validator_network_identity_pubkey,
        validator_network_address,
        full_node_network_identity_pubkey,
        full_node_network_address,
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_is_valid"></a>

## Function `is_valid`

Returns true if all of the following is true:
1) there is a ValidatorConfig resource under the address, and
2) the config is set, and
NB! currently we do not require the the operator_account to be set


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    exists&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr) && <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&borrow_global&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_config"></a>

## Function `get_config`

Get Config
Aborts if there is no ValidatorConfig resource of if its config is empty


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="#0x1_ValidatorConfig_Config">Config</a> <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(exists&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), EVALIDATOR_RESOURCE_DOES_NOT_EXIST);
    <b>let</b> config = &borrow_global&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config;
    *<a href="Option.md#0x1_Option_borrow">Option::borrow</a>(config)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_operator"></a>

## Function `get_operator`

Get operator's account
Aborts if there is no ValidatorConfig resource, if its operator_account is
empty, returns the input


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: address): address <b>acquires</b> <a href="#0x1_ValidatorConfig">ValidatorConfig</a> {
    <b>assert</b>(exists&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr), EVALIDATOR_RESOURCE_DOES_NOT_EXIST);
    <b>let</b> t_ref = borrow_global&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr);
    *<a href="Option.md#0x1_Option_borrow_with_default">Option::borrow_with_default</a>(&t_ref.operator_account, &addr)
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_consensus_pubkey"></a>

## Function `get_consensus_pubkey`

Get consensus_pubkey from Config
Never aborts


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.consensus_pubkey
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_validator_network_identity_pubkey"></a>

## Function `get_validator_network_identity_pubkey`

Get validator's network identity pubkey from Config
Never aborts


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.validator_network_identity_pubkey
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_get_validator_network_address"></a>

## Function `get_validator_network_address`

Get validator's network address from Config
Never aborts


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x1_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.validator_network_address
}
</code></pre>



</details>

<a name="0x1_ValidatorConfig_Specification"></a>

## Specification


<a name="0x1_ValidatorConfig_Specification_publish"></a>

### Function `publish`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_publish">publish</a>(account: &signer, lr_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
<b>aborts_if</b> <a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



Returns true if a ValidatorConfig resource exists under addr.


<a name="0x1_ValidatorConfig_spec_exists_config"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(addr: address): bool {
    exists&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr)
}
</code></pre>



<a name="0x1_ValidatorConfig_Specification_set_operator"></a>

### Function `set_operator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_operator">set_operator</a>(account: &signer, operator_account: address)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) == operator_account;
</code></pre>



Returns true if addr has an operator account.


<a name="0x1_ValidatorConfig_spec_has_operator"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account)
}
</code></pre>


Returns the operator account of a validator if it has one,
and returns the addr itself otherwise.


<a name="0x1_ValidatorConfig_spec_get_operator"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(addr: address): address {
    <b>if</b> (<a href="#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(addr)) {
        <a href="Option.md#0x1_Option_spec_value_inside">Option::spec_value_inside</a>(<b>global</b>&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).operator_account)
    } <b>else</b> {
        addr
    }
}
</code></pre>



<a name="0x1_ValidatorConfig_Specification_remove_operator"></a>

### Function `remove_operator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_remove_operator">remove_operator</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> !<a href="#0x1_ValidatorConfig_spec_has_operator">spec_has_operator</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account))
    == <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
</code></pre>



<a name="0x1_ValidatorConfig_Specification_set_config"></a>

### Function `set_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_set_config">set_config</a>(signer: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, full_node_network_identity_pubkey: vector&lt;u8&gt;, full_node_network_address: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer) != <a href="#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(validator_account);
<b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(validator_account);
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_spec_ed25519_validate_pubkey">Signature::spec_ed25519_validate_pubkey</a>(consensus_pubkey);
<b>ensures</b> <a href="#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(validator_account);
</code></pre>



Returns true if there a config published under addr.


<a name="0x1_ValidatorConfig_spec_has_config"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



<a name="0x1_ValidatorConfig_Specification_is_valid"></a>

### Function `is_valid`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x1_ValidatorConfig_spec_is_valid">spec_is_valid</a>(addr);
</code></pre>



Returns true if addr is a valid validator.


<a name="0x1_ValidatorConfig_spec_is_valid"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_is_valid">spec_is_valid</a>(addr: address): bool {
    <a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(addr) && <a href="#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(addr)
}
</code></pre>



<a name="0x1_ValidatorConfig_@Validator_stays_valid_once_it_becomes_valid"></a>

#### Validator stays valid once it becomes valid



<a name="0x1_ValidatorConfig_ValidatorStaysValid"></a>


<pre><code><b>schema</b> <a href="#0x1_ValidatorConfig_ValidatorStaysValid">ValidatorStaysValid</a> {
    <b>ensures</b> forall validator: address:
        <b>old</b>(<a href="#0x1_ValidatorConfig_spec_is_valid">spec_is_valid</a>(validator)) ==&gt; <a href="#0x1_ValidatorConfig_spec_is_valid">spec_is_valid</a>(validator);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_ValidatorConfig_ValidatorStaysValid">ValidatorStaysValid</a> <b>to</b> *;
</code></pre>



<a name="0x1_ValidatorConfig_Specification_get_config"></a>

### Function `get_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="#0x1_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(addr);
<b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_has_config">spec_has_config</a>(addr);
<b>ensures</b> result == <a href="#0x1_ValidatorConfig_spec_get_config">spec_get_config</a>(addr);
</code></pre>



Returns the config published under addr.


<a name="0x1_ValidatorConfig_spec_get_config"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorConfig_spec_get_config">spec_get_config</a>(addr: address): <a href="#0x1_ValidatorConfig_Config">Config</a> {
    <a href="Option.md#0x1_Option_spec_value_inside">Option::spec_value_inside</a>(<b>global</b>&lt;<a href="#0x1_ValidatorConfig">ValidatorConfig</a>&gt;(addr).config)
}
</code></pre>



<a name="0x1_ValidatorConfig_Specification_get_operator"></a>

### Function `get_operator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorConfig_get_operator">get_operator</a>(addr: address): address
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_ValidatorConfig_spec_exists_config">spec_exists_config</a>(addr);
<b>ensures</b> result == <a href="#0x1_ValidatorConfig_spec_get_operator">spec_get_operator</a>(addr);
</code></pre>




<pre><code>pragma verify = <b>true</b>, aborts_if_is_strict = <b>true</b>;
</code></pre>
