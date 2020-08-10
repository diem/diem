
<a name="0x1_ValidatorOperatorConfig"></a>

# Module `0x1::ValidatorOperatorConfig`

### Table of Contents

-  [Resource `ValidatorOperatorConfig`](#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig)
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
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_validator_operator_role">Roles::has_validator_operator_role</a>(account), ENO_VALIDATOR_OPERATOR_ROLE);
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
    <b>assert</b>(exists&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr), EVALIDATOR_OPERATOR_RESOURCE_DOES_NOT_EXIST);
    <b>let</b> t_ref = borrow_global&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr);
    *&t_ref.human_name
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




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
<b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_validator_operator_role_addr">Roles::spec_has_validator_operator_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <a href="#0x1_ValidatorOperatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_ValidatorOperatorConfig_spec_exists_config">spec_exists_config</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



Returns true if a ValidatorOperatorConfig resource exists under addr.


<a name="0x1_ValidatorOperatorConfig_spec_exists_config"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorOperatorConfig_spec_exists_config">spec_exists_config</a>(addr: address): bool {
    exists&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr)
}
</code></pre>


Returns the human name of the validator


<a name="0x1_ValidatorOperatorConfig_spec_get_human_name"></a>


<pre><code><b>define</b> <a href="#0x1_ValidatorOperatorConfig_spec_get_human_name">spec_get_human_name</a>(addr: address): vector&lt;u8&gt; {
    <b>global</b>&lt;<a href="#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(addr).human_name
}
</code></pre>



<a name="0x1_ValidatorOperatorConfig_Specification_get_human_name"></a>

### Function `get_human_name`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> !<a href="#0x1_ValidatorOperatorConfig_spec_exists_config">spec_exists_config</a>(addr);
<b>ensures</b> result == <a href="#0x1_ValidatorOperatorConfig_spec_get_human_name">spec_get_human_name</a>(addr);
</code></pre>



<a name="0x1_ValidatorOperatorConfig_Specification_has_validator_operator_config"></a>

### Function `has_validator_operator_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(addr: address): bool
</code></pre>




<pre><code><b>ensures</b> result == <a href="#0x1_ValidatorOperatorConfig_spec_exists_config">spec_exists_config</a>(addr);
</code></pre>
