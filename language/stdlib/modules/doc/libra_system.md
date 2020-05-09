
<a name="0x0_LibraSystem"></a>

# Module `0x0::LibraSystem`

### Table of Contents

-  [Struct `ValidatorInfo`](#0x0_LibraSystem_ValidatorInfo)
-  [Struct `CapabilityHolder`](#0x0_LibraSystem_CapabilityHolder)
-  [Struct `T`](#0x0_LibraSystem_T)
-  [Function `get_validator_config`](#0x0_LibraSystem_get_validator_config)
-  [Function `initialize_validator_set`](#0x0_LibraSystem_initialize_validator_set)
-  [Function `get_validator_set`](#0x0_LibraSystem_get_validator_set)
-  [Function `set_validator_set`](#0x0_LibraSystem_set_validator_set)
-  [Function `add_validator`](#0x0_LibraSystem_add_validator)
-  [Function `remove_validator`](#0x0_LibraSystem_remove_validator)
-  [Function `update_and_reconfigure`](#0x0_LibraSystem_update_and_reconfigure)
-  [Function `is_validator`](#0x0_LibraSystem_is_validator)
-  [Function `get_validator_info`](#0x0_LibraSystem_get_validator_info)
-  [Function `validator_set_size`](#0x0_LibraSystem_validator_set_size)
-  [Function `get_ith_validator_address`](#0x0_LibraSystem_get_ith_validator_address)
-  [Function `is_sender_authorized_`](#0x0_LibraSystem_is_sender_authorized_)
-  [Function `get_validator_index_`](#0x0_LibraSystem_get_validator_index_)
-  [Function `update_ith_validator_info_`](#0x0_LibraSystem_update_ith_validator_info_)
-  [Function `is_validator_`](#0x0_LibraSystem_is_validator_)



<a name="0x0_LibraSystem_ValidatorInfo"></a>

## Struct `ValidatorInfo`



<pre><code><b>struct</b> <a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a>
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

<code>config: <a href="validator_config.md#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraSystem_CapabilityHolder"></a>

## Struct `CapabilityHolder`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="libra_configs.md#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x0_LibraSystem_T">LibraSystem::T</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraSystem_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_LibraSystem_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>scheme: u8</code>
</dt>
<dd>

</dd>
<dt>

<code>validators: vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraSystem_get_validator_config"></a>

## Function `get_validator_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_config">get_validator_config</a>(addr: address): <a href="validator_config.md#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_config">get_validator_config</a>(addr: address): <a href="validator_config.md#0x0_ValidatorConfig_Config">ValidatorConfig::Config</a> {
    <b>let</b> validator_set = <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>();
    <b>let</b> validator_index_vec = <a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(&validator_set.validators, addr);
    *&(*<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&validator_set.validators, *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&validator_index_vec, 0))).config
}
</code></pre>



</details>

<a name="0x0_LibraSystem_initialize_validator_set"></a>

## Function `initialize_validator_set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_initialize_validator_set">initialize_validator_set</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_initialize_validator_set">initialize_validator_set</a>(config_account: &signer) {
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(config_account) == <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(), 1);

    <b>let</b> cap = <a href="libra_configs.md#0x0_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>&lt;<a href="#0x0_LibraSystem_T">T</a>&gt;(
        <a href="#0x0_LibraSystem_T">T</a> {
            scheme: 0,
            validators: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>(),
        },
        config_account
    );
    move_to(config_account, <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a> { cap })
}
</code></pre>



</details>

<a name="0x0_LibraSystem_get_validator_set"></a>

## Function `get_validator_set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>(): <a href="#0x0_LibraSystem_T">LibraSystem::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>(): <a href="#0x0_LibraSystem_T">T</a> {
    <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_LibraSystem_T">T</a>&gt;()
}
</code></pre>



</details>

<a name="0x0_LibraSystem_set_validator_set"></a>

## Function `set_validator_set`



<pre><code><b>fun</b> <a href="#0x0_LibraSystem_set_validator_set">set_validator_set</a>(value: <a href="#0x0_LibraSystem_T">LibraSystem::T</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraSystem_set_validator_set">set_validator_set</a>(value: <a href="#0x0_LibraSystem_T">T</a>) <b>acquires</b> <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    <a href="libra_configs.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>&lt;<a href="#0x0_LibraSystem_T">T</a>&gt;(&borrow_global&lt;<a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a>&gt;(<a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>()).cap, value)
}
</code></pre>



</details>

<a name="0x0_LibraSystem_add_validator"></a>

## Function `add_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_add_validator">add_validator</a>(account_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_add_validator">add_validator</a>(account_address: address) <b>acquires</b> <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    Transaction::assert(Transaction::sender() == 0xA550C18, 1);
    // A prospective validator must have a validator config <b>resource</b>
    Transaction::assert(<a href="validator_config.md#0x0_ValidatorConfig_has">ValidatorConfig::has</a>(account_address), 17);

    <b>let</b> validator_set = <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>();
    // Ensure that this address is not already a validator
    Transaction::assert(!<a href="#0x0_LibraSystem_is_validator_">is_validator_</a>(account_address, &validator_set.validators), 18);

    <b>let</b> config = <a href="validator_config.md#0x0_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(account_address);
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> validator_set.validators, <a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a> {
        addr: account_address,
        config, // <b>copy</b> the config over <b>to</b> ValidatorSet
        consensus_voting_power: 1,
    });

    <a href="#0x0_LibraSystem_set_validator_set">set_validator_set</a>(validator_set);
}
</code></pre>



</details>

<a name="0x0_LibraSystem_remove_validator"></a>

## Function `remove_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_remove_validator">remove_validator</a>(account_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_remove_validator">remove_validator</a>(account_address: address) <b>acquires</b> <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    Transaction::assert(Transaction::sender() == 0xA550C18, 1);
    <b>let</b> validator_set = <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>();

    // Ensure that this address is an active validator
    <b>let</b> to_remove_index_vec = <a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(&validator_set.validators, account_address);
    Transaction::assert(!<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&to_remove_index_vec), 21);
    <b>let</b> to_remove_index = *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&to_remove_index_vec, 0);
    // Remove corresponding <a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a> from the validator set
    _  = <a href="vector.md#0x0_Vector_swap_remove">Vector::swap_remove</a>(&<b>mut</b> validator_set.validators, to_remove_index);

    <a href="#0x0_LibraSystem_set_validator_set">set_validator_set</a>(validator_set);
}
</code></pre>



</details>

<a name="0x0_LibraSystem_update_and_reconfigure"></a>

## Function `update_and_reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_update_and_reconfigure">update_and_reconfigure</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_update_and_reconfigure">update_and_reconfigure</a>() <b>acquires</b> <a href="#0x0_LibraSystem_CapabilityHolder">CapabilityHolder</a> {
    Transaction::assert(<a href="#0x0_LibraSystem_is_sender_authorized_">is_sender_authorized_</a>(), 22);

    <b>let</b> validator_set = <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>();
    <b>let</b> validators = &<b>mut</b> validator_set.validators;

    <b>let</b> size = <a href="vector.md#0x0_Vector_length">Vector::length</a>(validators);
    <b>if</b> (size == 0) {
        <b>return</b>
    };

    <b>let</b> i = 0;
    <b>let</b> configs_changed = <b>false</b>;
    <b>while</b> (i &lt; size) {
        <b>let</b> validator_info_update = <a href="#0x0_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators, i);

        configs_changed = configs_changed || validator_info_update;
        i = i + 1;
    };
    <b>if</b> (configs_changed) {
        <a href="#0x0_LibraSystem_set_validator_set">set_validator_set</a>(validator_set);
    };
}
</code></pre>



</details>

<a name="0x0_LibraSystem_is_validator"></a>

## Function `is_validator`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_is_validator">is_validator</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_is_validator">is_validator</a>(addr: address): bool {
    <a href="#0x0_LibraSystem_is_validator_">is_validator_</a>(addr, &<a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>().validators)
}
</code></pre>



</details>

<a name="0x0_LibraSystem_get_validator_info"></a>

## Function `get_validator_info`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_info">get_validator_info</a>(addr: address): <a href="#0x0_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_validator_info">get_validator_info</a>(addr: address): <a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a> {
    <b>let</b> validator_set = <a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>();
    <b>let</b> validator_index_vec = <a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(&validator_set.validators, addr);
    Transaction::assert(!<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&validator_index_vec), 19);

    *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&validator_set.validators, *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&validator_index_vec, 0))
}
</code></pre>



</details>

<a name="0x0_LibraSystem_validator_set_size"></a>

## Function `validator_set_size`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_validator_set_size">validator_set_size</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_validator_set_size">validator_set_size</a>(): u64 {
    <a href="vector.md#0x0_Vector_length">Vector::length</a>(&<a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>().validators)
}
</code></pre>



</details>

<a name="0x0_LibraSystem_get_ith_validator_address"></a>

## Function `get_ith_validator_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraSystem_get_ith_validator_address">get_ith_validator_address</a>(i: u64): address {
    <a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(&<a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>().validators, i).addr
}
</code></pre>



</details>

<a name="0x0_LibraSystem_is_sender_authorized_"></a>

## Function `is_sender_authorized_`



<pre><code><b>fun</b> <a href="#0x0_LibraSystem_is_sender_authorized_">is_sender_authorized_</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraSystem_is_sender_authorized_">is_sender_authorized_</a>(): bool {
    // succeed fast
    <b>if</b> (Transaction::sender() == 0xA550C18 || Transaction::sender() == 0x0) {
        <b>return</b> <b>true</b>
    };
    <b>let</b> validators = &<a href="#0x0_LibraSystem_get_validator_set">get_validator_set</a>().validators;
    // scan the validators <b>to</b> find a match
    <b>let</b> size = <a href="vector.md#0x0_Vector_length">Vector::length</a>(validators);
    // always <b>true</b>: size &gt; 3 (see remove_validator code)

    <b>let</b> i = 0;
    <b>while</b> (i &lt; size) {
        <b>if</b> (<a href="validator_config.md#0x0_ValidatorConfig_get_validator_operator_account">ValidatorConfig::get_validator_operator_account</a>(<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(validators, i).addr) ==
            Transaction::sender()) {
            <b>return</b> <b>true</b>
        };
        i = i + 1;
    };
    <b>return</b> <b>false</b>
}
</code></pre>



</details>

<a name="0x0_LibraSystem_get_validator_index_"></a>

## Function `get_validator_index_`



<pre><code><b>fun</b> <a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;, addr: address): vector&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(validators: &vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;, addr: address): vector&lt;u64&gt; {
    <b>let</b> size = <a href="vector.md#0x0_Vector_length">Vector::length</a>(validators);
    <b>let</b> result: vector&lt;u64&gt; = <a href="vector.md#0x0_Vector_empty">Vector::empty</a>();
    <b>if</b> (size == 0) {
        <b>return</b> result
    };

    <b>let</b> i = 0;
    <b>while</b> (i &lt; size) {
        <b>let</b> validator_info_ref = <a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(validators, i);
        <b>if</b> (validator_info_ref.addr == addr) {
            <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> result, i);
            <b>return</b> result
        };
        i = i + 1;
    };

    result
}
</code></pre>



</details>

<a name="0x0_LibraSystem_update_ith_validator_info_"></a>

## Function `update_ith_validator_info_`



<pre><code><b>fun</b> <a href="#0x0_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;, i: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraSystem_update_ith_validator_info_">update_ith_validator_info_</a>(validators: &<b>mut</b> vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;, i: u64): bool {
    <b>let</b> size = <a href="vector.md#0x0_Vector_length">Vector::length</a>(validators);
    <b>if</b> (i &gt;= size) {
        <b>return</b> <b>false</b>
    };
    <b>let</b> validator_info = <a href="vector.md#0x0_Vector_borrow_mut">Vector::borrow_mut</a>(validators, i);
    <b>let</b> new_validator_config = <a href="validator_config.md#0x0_ValidatorConfig_get_config">ValidatorConfig::get_config</a>(validator_info.addr);
    // check <b>if</b> information is the same
    <b>let</b> config_ref = &<b>mut</b> validator_info.config;

    <b>if</b> (config_ref == &new_validator_config) {
        <b>return</b> <b>false</b>
    };
    *config_ref = *&new_validator_config;

    <b>true</b>
}
</code></pre>



</details>

<a name="0x0_LibraSystem_is_validator_"></a>

## Function `is_validator_`



<pre><code><b>fun</b> <a href="#0x0_LibraSystem_is_validator_">is_validator_</a>(addr: address, validators_vec_ref: &vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">LibraSystem::ValidatorInfo</a>&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraSystem_is_validator_">is_validator_</a>(addr: address, validators_vec_ref: &vector&lt;<a href="#0x0_LibraSystem_ValidatorInfo">ValidatorInfo</a>&gt;): bool {
    !<a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&<a href="#0x0_LibraSystem_get_validator_index_">get_validator_index_</a>(validators_vec_ref, addr))
}
</code></pre>



</details>
