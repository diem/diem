
<a name="0x1_ValidatorOperatorConfig"></a>

# Module `0x1::ValidatorOperatorConfig`

Stores the string name of a ValidatorOperator account.


-  [Resource `ValidatorOperatorConfigChainMarker`](#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker)
-  [Resource `ValidatorOperatorConfig`](#0x1_ValidatorOperatorConfig_ValidatorOperatorConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_ValidatorOperatorConfig_initialize)
-  [Function `publish`](#0x1_ValidatorOperatorConfig_publish)
-  [Function `get_human_name`](#0x1_ValidatorOperatorConfig_get_human_name)
-  [Function `has_validator_operator_config`](#0x1_ValidatorOperatorConfig_has_validator_operator_config)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker"></a>

## Resource `ValidatorOperatorConfigChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker">ValidatorOperatorConfigChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_ValidatorOperatorConfig_ValidatorOperatorConfig"></a>

## Resource `ValidatorOperatorConfig`



<pre><code><b>struct</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> <b>has</b> key
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ValidatorOperatorConfig_ECHAIN_MARKER"></a>

The <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker">ValidatorOperatorConfigChainMarker</a></code> resource was not in the required state


<pre><code><b>const</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 9;
</code></pre>



<a name="0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG"></a>

The <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a></code> was not in the required state


<pre><code><b>const</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_ValidatorOperatorConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_initialize">initialize</a>&lt;T&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_initialize">initialize</a>&lt;T&gt;(account: &signer) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker">ValidatorOperatorConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );
    <b>move_to</b>(account, <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker">ValidatorOperatorConfigChainMarker</a>&lt;T&gt;{});
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">publish</a>&lt;T&gt;(validator_operator_account: &signer, human_name: vector&lt;u8&gt;, _cap: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">publish</a>&lt;T&gt;(
    validator_operator_account: &signer,
    human_name: vector&lt;u8&gt;,
    _cap: Cap&lt;T&gt;
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ValidatorOperatorConfigChainMarker">ValidatorOperatorConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(validator_operator_account)),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>)
    );

    <b>move_to</b>(validator_operator_account, <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
        human_name,
    });
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_get_human_name"></a>

## Function `get_human_name`

Get validator's account human name
Aborts if there is no ValidatorOperatorConfig resource


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(validator_operator_addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_get_human_name">get_human_name</a>(validator_operator_addr: <b>address</b>): vector&lt;u8&gt; <b>acquires</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a> {
    <b>assert</b>!(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_EVALIDATOR_OPERATOR_CONFIG">EVALIDATOR_OPERATOR_CONFIG</a>));
    *&<b>borrow_global</b>&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(validator_operator_addr).human_name
}
</code></pre>



</details>

<a name="0x1_ValidatorOperatorConfig_has_validator_operator_config"></a>

## Function `has_validator_operator_config`



<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">has_validator_operator_config</a>(validator_operator_addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a>&gt;(validator_operator_addr)
}
</code></pre>



</details>
