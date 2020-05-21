
<a name="0x0_ValidatorConfig"></a>

# Module `0x0::ValidatorConfig`

### Table of Contents

-  [Struct `Config`](#0x0_ValidatorConfig_Config)
-  [Struct `T`](#0x0_ValidatorConfig_T)
-  [Function `has`](#0x0_ValidatorConfig_has)
-  [Function `get_config`](#0x0_ValidatorConfig_get_config)
-  [Function `get_consensus_pubkey`](#0x0_ValidatorConfig_get_consensus_pubkey)
-  [Function `get_validator_operator_account`](#0x0_ValidatorConfig_get_validator_operator_account)
-  [Function `register_candidate_validator`](#0x0_ValidatorConfig_register_candidate_validator)
-  [Function `set_delegated_account`](#0x0_ValidatorConfig_set_delegated_account)
-  [Function `remove_delegated_account`](#0x0_ValidatorConfig_remove_delegated_account)
-  [Function `rotate_consensus_pubkey`](#0x0_ValidatorConfig_rotate_consensus_pubkey)
-  [Function `rotate_consensus_pubkey_of_sender`](#0x0_ValidatorConfig_rotate_consensus_pubkey_of_sender)
-  [Function `get_validator_network_identity_pubkey`](#0x0_ValidatorConfig_get_validator_network_identity_pubkey)
-  [Function `get_validator_network_address`](#0x0_ValidatorConfig_get_validator_network_address)
-  [Function `rotate_validator_network_identity_pubkey`](#0x0_ValidatorConfig_rotate_validator_network_identity_pubkey)
-  [Function `rotate_validator_network_address`](#0x0_ValidatorConfig_rotate_validator_network_address)



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

<code>validator_network_signing_pubkey: vector&lt;u8&gt;</code>
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

<code>config: <a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a></code>
</dt>
<dd>

</dd>
<dt>

<code>delegated_account: <a href="option.md#0x0_Option_T">Option::T</a>&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_ValidatorConfig_has"></a>

## Function `has`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_has">has</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_has">has</a>(addr: address): bool {
    exists&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr)
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
    *&borrow_global&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr).config
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_consensus_pubkey"></a>

## Function `get_consensus_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_consensus_pubkey">get_consensus_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): vector&lt;u8&gt; {
    *&config_ref.consensus_pubkey
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_validator_operator_account"></a>

## Function `get_validator_operator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_operator_account">get_validator_operator_account</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_operator_account">get_validator_operator_account</a>(addr: address): address <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <a href="option.md#0x0_Option_get_with_default">Option::get_with_default</a>(&borrow_global&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(addr).delegated_account, addr)
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_register_candidate_validator"></a>

## Function `register_candidate_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_register_candidate_validator">register_candidate_validator</a>(consensus_pubkey: vector&lt;u8&gt;, validator_network_signing_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, full_node_network_identity_pubkey: vector&lt;u8&gt;, full_node_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_register_candidate_validator">register_candidate_validator</a>(
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_signing_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    full_node_network_identity_pubkey: vector&lt;u8&gt;,
    full_node_network_address: vector&lt;u8&gt;) {

    move_to_sender&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(
        <a href="#0x0_ValidatorConfig_T">T</a> {
            config: <a href="#0x0_ValidatorConfig_Config">Config</a> {
                consensus_pubkey: consensus_pubkey,
                validator_network_signing_pubkey,
                validator_network_identity_pubkey,
                validator_network_address,
                full_node_network_identity_pubkey,
                full_node_network_address,
            },
            delegated_account: <a href="option.md#0x0_Option_none">Option::none</a>()
        }
    );
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_set_delegated_account"></a>

## Function `set_delegated_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_delegated_account">set_delegated_account</a>(delegated_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_set_delegated_account">set_delegated_account</a>(delegated_account: address) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    Transaction::assert(<a href="libra_account.md#0x0_LibraAccount_exists">LibraAccount::exists</a>(delegated_account), 5);
    // check delegated address is different from transaction's sender
    Transaction::assert(delegated_account != Transaction::sender(), 6);
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(Transaction::sender());
    t_ref.delegated_account = <a href="option.md#0x0_Option_some">Option::some</a>(delegated_account)
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_remove_delegated_account"></a>

## Function `remove_delegated_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_remove_delegated_account">remove_delegated_account</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_remove_delegated_account">remove_delegated_account</a>() <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(Transaction::sender());
    t_ref.delegated_account = <a href="option.md#0x0_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_rotate_consensus_pubkey"></a>

## Function `rotate_consensus_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_consensus_pubkey">rotate_consensus_pubkey</a>(validator_account: address, new_consensus_pubkey: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_consensus_pubkey">rotate_consensus_pubkey</a>(
    validator_account: address,
    new_consensus_pubkey: vector&lt;u8&gt;,
    // _proof: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <b>let</b> addr = <a href="#0x0_ValidatorConfig_get_validator_operator_account">get_validator_operator_account</a>(validator_account);
    Transaction::assert(Transaction::sender() == addr, 1);

    // TODO(valerini): verify the proof of posession of new_consensus_secretkey

    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(validator_account);
    // Set the new key
    t_ref.config.consensus_pubkey = new_consensus_pubkey;
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_rotate_consensus_pubkey_of_sender"></a>

## Function `rotate_consensus_pubkey_of_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_consensus_pubkey_of_sender">rotate_consensus_pubkey_of_sender</a>(new_consensus_pubkey: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_consensus_pubkey_of_sender">rotate_consensus_pubkey_of_sender</a>(new_consensus_pubkey: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <a href="#0x0_ValidatorConfig_rotate_consensus_pubkey">rotate_consensus_pubkey</a>(Transaction::sender(), new_consensus_pubkey);
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_validator_network_identity_pubkey"></a>

## Function `get_validator_network_identity_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_identity_pubkey">get_validator_network_identity_pubkey</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): vector&lt;u8&gt; {
    *&config_ref.validator_network_identity_pubkey
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_get_validator_network_address"></a>

## Function `get_validator_network_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_get_validator_network_address">get_validator_network_address</a>(config_ref: &<a href="#0x0_ValidatorConfig_Config">Config</a>): vector&lt;u8&gt; {
    *&config_ref.validator_network_address
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_rotate_validator_network_identity_pubkey"></a>

## Function `rotate_validator_network_identity_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_validator_network_identity_pubkey">rotate_validator_network_identity_pubkey</a>(validator_account: address, validator_network_identity_pubkey: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_validator_network_identity_pubkey">rotate_validator_network_identity_pubkey</a>(
    validator_account: address,
    validator_network_identity_pubkey: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <b>let</b> addr = <a href="#0x0_ValidatorConfig_get_validator_operator_account">get_validator_operator_account</a>(validator_account);
    Transaction::assert(Transaction::sender() == addr, 1);

    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(validator_account);
    t_ref.config.validator_network_identity_pubkey = *&validator_network_identity_pubkey;
}
</code></pre>



</details>

<a name="0x0_ValidatorConfig_rotate_validator_network_address"></a>

## Function `rotate_validator_network_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_validator_network_address">rotate_validator_network_address</a>(validator_account: address, validator_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_ValidatorConfig_rotate_validator_network_address">rotate_validator_network_address</a>(
    validator_account: address,
    validator_network_address: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_ValidatorConfig_T">T</a> {
    <b>let</b> addr = <a href="#0x0_ValidatorConfig_get_validator_operator_account">get_validator_operator_account</a>(validator_account);
    Transaction::assert(Transaction::sender() == addr, 1);

    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_ValidatorConfig_T">T</a>&gt;(validator_account);
    t_ref.config.validator_network_address = validator_network_address;
}
</code></pre>



</details>
