
<a name="0x1_ValidatorOperatorConfig"></a>

# Module `0x1::ValidatorOperatorConfig`



-  [Resource <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a></code>](#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig)
-  [Const <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a></code>](#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG)
-  [Function <code>publish</code>](#0x1_ValidatorOperatorConfig_publish)
-  [Function <code>get_human_name</code>](#0x1_ValidatorOperatorConfig_get_human_name)
-  [Function <code>has_validator_operator_config</code>](#0x1_ValidatorOperatorConfig_has_validator_operator_config)


<a name="0x1_ValidatorOperatorConfig_ValidatorOperatorConfig"></a>

## Resource `ValidatorOperatorConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>
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

The <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a></code> was not in the required state


<pre><code><b>const</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_ValidatorOperatorConfig_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">publish</a>(validator_operator_account: &signer, lr_account: &signer, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">publish</a>(
    validator_operator_account: &signer,
    lr_account: &signer,
    human_name: vector&lt;u8&gt;,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="Roles.md#0x1_Roles_assert_validator_operator">Roles::assert_validator_operator</a>(validator_operator_account);
    <b>assert</b>(
        !<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>)
    );

    move_to(validator_operator_account, <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
        human_name,
    });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidatorOperator">Roles::AbortsIfNotValidatorOperator</a>{validator_operator_addr: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account)};
<b>include</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_PublishAbortsIf">PublishAbortsIf</a> {validator_operator_addr: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_operator_account)};
<b>ensures</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(validator_operator_account));
</code></pre>




<a name="0x1_ValidatorOperatorConfig_PublishAbortsIf"></a>


<pre><code><b>schema</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_PublishAbortsIf">PublishAbortsIf</a> {
    validator_operator_addr: address;
    lr_account: signer;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotValidatorOperator">Roles::AbortsIfNotValidatorOperator</a>;
    <b>aborts_if</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr)
        <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_get_human_name"></a>

## Function `get_human_name`

Get validator's account human name
Aborts if there is no ValidatorOperatorConfig resource


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(validator_operator_addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(validator_operator_addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
    <b>assert</b>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>));
    *&borrow_global&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(validator_operator_addr).human_name
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>aborts_if</b> !<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>ensures</b> result == <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(validator_operator_addr);
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_has_validator_operator_config"></a>

## Function `has_validator_operator_config`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr: address): bool {
    <b>exists</b>&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(validator_operator_addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>ensures</b> result == <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr);
</code></pre>


If address has a ValidatorOperatorConfig, it has a validator operator role.
This invariant is useful in LibraSystem so we don't have to check whether
every validator address has a validator role.


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1:address <b>where</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr1):
    <a href="Roles.md#0x1_Roles_spec_has_validator_operator_role_addr">Roles::spec_has_validator_operator_role_addr</a>(addr1);
</code></pre>



</details>

[]: # (File containing markdown style reference definitions to be included in each generated doc)
