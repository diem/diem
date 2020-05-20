
<a name="0x0_ValidatorConfig"></a>

# Module `0x0::ValidatorConfig`

### Table of Contents

-  [Struct `Config`](#0x0_ValidatorConfig_Config)
-  [Struct `T`](#0x0_ValidatorConfig_T)
-  [Function `publish`](#0x0_ValidatorConfig_publish)
-  [Function `set_operator`](#0x0_ValidatorConfig_set_operator)
-  [Function `remove_operator`](#0x0_ValidatorConfig_remove_operator)
-  [Function `set_config`](#0x0_ValidatorConfig_set_config)
-  [Function `set_consensus_pubkey`](#0x0_ValidatorConfig_set_consensus_pubkey)
-  [Function `is_valid`](#0x0_ValidatorConfig_is_valid)
-  [Function `get_config`](#0x0_ValidatorConfig_get_config)
-  [Function `get_operator`](#0x0_ValidatorConfig_get_operator)
-  [Function `get_consensus_pubkey`](#0x0_ValidatorConfig_get_consensus_pubkey)
-  [Function `get_validator_network_identity_pubkey`](#0x0_ValidatorConfig_get_validator_network_identity_pubkey)
-  [Function `get_validator_network_address`](#0x0_ValidatorConfig_get_validator_network_address)



<a name="0x0_ValidatorConfig_Config"></a>

## Struct `Config`



<pre><code><b>struct</b> <a href="#0x0_ValidatorConfig_Config">Config</a>
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

<a name="0x0_ValidatorConfig_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_ValidatorConfig_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>config: <a href="option.md#0x0_Option_T">Option::T</a>&lt;<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>operator_account: <a href="option.md#0x0_Option_T">Option::T</a>&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_ValidatorConfig_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_publish">publish</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_publish">publish</a>(signer: &signer) {
    Transaction::assert(Transaction::sender() == 0xA550C18, 1101);
    move_to(signer, <a href="#0x0_ValidatorConfig_T">T</a> {
        config: <a href="option.md#0x0_Option_none">Option::none</a>(),
        operator_account: <a href="option.md#0x0_Option_none">Option::none</a>(),
    });
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_set_operator"></a>

## Function `set_operator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_operator">set_operator</a>(operator_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_operator">set_operator</a>(operator_account: address) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    (borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(Transaction::sender())).operator_account = <a href="option.md#0x0_Option_some">Option::some</a>(operator_account);
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_remove_operator"></a>

## Function `remove_operator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_remove_operator">remove_operator</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_remove_operator">remove_operator</a>() <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    // <a href="#0x0_ValidatorConfig_Config">Config</a> field remains set
    (borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(Transaction::sender())).operator_account = <a href="option.md#0x0_Option_none">Option::none</a>();
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_set_config"></a>

## Function `set_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_config">set_config</a>(signer: &signer, validator_account: address, consensus_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, full_node_network_identity_pubkey: vector&lt;u8&gt;, full_node_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_config">set_config</a>(
    signer: &signer,
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    full_node_network_identity_pubkey: vector&lt;u8&gt;,
    full_node_network_address: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(signer) ==
                        <a href="#0x0_ValidatorConfig_get_operator">get_operator</a>(validator_account), 1101);
    // TODO(valerini): verify the validity of new_config.consensus_pubkey and
    // the proof of posession
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(validator_account);
    t_ref.config = <a href="option.md#0x0_Option_some">Option::some</a>(<a href="#0x0_ValidatorConfig_Config">Config</a> {
        consensus_pubkey,
        validator_network_identity_pubkey,
        validator_network_address,
        full_node_network_identity_pubkey,
        full_node_network_address,
    });
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_set_consensus_pubkey"></a>

## Function `set_consensus_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_consensus_pubkey">set_consensus_pubkey</a>(validator_account: address, consensus_pubkey: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_consensus_pubkey">set_consensus_pubkey</a>(
    validator_account: address,
    consensus_pubkey: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    Transaction::assert(Transaction::sender() ==
                        <a href="#0x0_ValidatorConfig_get_operator">get_operator</a>(validator_account), 1101);
    <b>let</b> t_config_ref = <a href="option.md#0x0_Option_borrow_mut">Option::borrow_mut</a>(&<b>mut</b> borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(validator_account).config);
    t_config_ref.consensus_pubkey = consensus_pubkey;
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_is_valid"></a>

## Function `is_valid`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_is_valid">is_valid</a>(addr: address): bool <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    exists&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr) && <a href="option.md#0x0_Option_is_some">Option::is_some</a>(&borrow_global&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr).config)
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_config"></a>

## Function `get_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_config">get_config</a>(addr: address): <a href="#0x0_ValidatorConfig_Config">Config</a> <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    Transaction::assert(exists&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr), 1106);
    <b>let</b> config = &borrow_global&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr).config;
    *<a href="option.md#0x0_Option_borrow">Option::borrow</a>(config)
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_operator"></a>

## Function `get_operator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_operator">get_operator</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_operator">get_operator</a>(addr: address): address <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    Transaction::assert(exists&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr), 1106);
    <b>let</b> t_ref = borrow_global&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr);
    *<a href="option.md#0x0_Option_borrow_with_default">Option::borrow_with_default</a>(&t_ref.operator_account, &addr)
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_consensus_pubkey"></a>

## Function `get_consensus_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.consensus_pubkey
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_validator_network_identity_pubkey"></a>

## Function `get_validator_network_identity_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.validator_network_identity_pubkey
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_validator_network_address"></a>

## Function `get_validator_network_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): &vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): &vector&lt;u8&gt; {
    &config_ref.validator_network_address
}
</code></pre>



</details>
