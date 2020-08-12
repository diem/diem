
<a name="0x1_ValidatorOperatorConfig"></a>

# Module `0x1::ValidatorOperatorConfig`

### Table of Contents

-  [Resource `ValidatorOperatorConfig`](#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig)
-  [Const `EVALIDATOR_OPERATOR_CONFIG`](#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG)
-  [Function `publish`](#0x1_ValidatorOperatorConfig_publish)
-  [Function `get_human_name`](#0x1_ValidatorOperatorConfig_get_human_name)
-  [Function `has_validator_operator_config`](#0x1_ValidatorOperatorConfig_has_validator_operator_config)
-  [Specification](#0x1_ValidatorOperatorConfig_Specification)
    -  [Function `publish`](#0x1_ValidatorOperatorConfig_Specification_publish)
    -  [Function `get_human_name`](#0x1_ValidatorOperatorConfig_Specification_get_human_name)
    -  [Function `has_validator_operator_config`](#0x1_ValidatorOperatorConfig_Specification_has_validator_operator_config)



<a name="0x1_ValidatorOperatorConfig_ValidatorOperatorConfig"></a>

## Resource `ValidatorOperatorConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>human_name: vector&lt;u8&gt;</code>
</dt>
<dd>
 The human readable name of this entity. Immutable.
</dd>
</dl>


</details>

<a name="0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG"></a>

## Const `EVALIDATOR_OPERATOR_CONFIG`



<pre><code><b>const</b> EVALIDATOR_OPERATOR_CONFIG: u64 = 0;
</code></pre>



<a name="0x1_ValidatorOperatorConfig_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_publish">publish</a>(account: &signer, lr_account: &signer, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_publish">publish</a>(
    account: &signer,
    lr_account: &signer,
    human_name: vector&lt;u8&gt;,
) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="Roles.md#0x1_Roles_assert_validator_operator">Roles::assert_validator_operator</a>(account);
    <b>assert</b>(
        !<a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EVALIDATOR_OPERATOR_CONFIG)
    );

    move_to(account, <a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
        human_name,
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_get_human_name"></a>

## Function `get_human_name`

Get validator's account human name
Aborts if there is no ValidatorOperatorConfig resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
    <b>assert</b>(<a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EVALIDATOR_OPERATOR_CONFIG));
    *&borrow_global&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr).human_name
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_has_validator_operator_config"></a>

## Function `has_validator_operator_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr: address): bool {
    exists&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_Specification"></a>

## Specification


<a name="0x1_ValidatorOperatorConfig_Specification_publish"></a>

### Function `publish`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_publish">publish</a>(account: &signer, lr_account: &signer, human_name: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidatorOperator">Roles::AbortsIfNotValidatorOperator</a>;
<b>aborts_if</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::ALREADY_PUBLISHED;
<b>ensures</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="0x1_ValidatorOperatorConfig_Specification_get_human_name"></a>

### Function `get_human_name`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>




<pre><code>pragma opaque;
<b>aborts_if</b> !<a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr) with Errors::NOT_PUBLISHED;
<b>ensures</b> result == <a href="#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(addr);
</code></pre>



<a name="0x1_ValidatorOperatorConfig_Specification_has_validator_operator_config"></a>

### Function `has_validator_operator_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr: address): bool
</code></pre>




<pre><code><b>ensures</b> result == <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr);
</code></pre>
