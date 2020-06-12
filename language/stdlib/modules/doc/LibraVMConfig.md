
<a name="0x1_LibraVMConfig"></a>

# Module `0x1::LibraVMConfig`

### Table of Contents

-  [Struct `LibraVMConfig`](#0x1_LibraVMConfig_LibraVMConfig)
-  [Struct `GasSchedule`](#0x1_LibraVMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_LibraVMConfig_GasConstants)
-  [Function `initialize`](#0x1_LibraVMConfig_initialize)
-  [Function `set_publishing_option`](#0x1_LibraVMConfig_set_publishing_option)



<a name="0x1_LibraVMConfig_LibraVMConfig"></a>

## Struct `LibraVMConfig`



<pre><code><b>struct</b> <a href="#0x1_LibraVMConfig">LibraVMConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>publishing_option: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>gas_schedule: <a href="#0x1_LibraVMConfig_GasSchedule">LibraVMConfig::GasSchedule</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_GasSchedule"></a>

## Struct `GasSchedule`



<pre><code><b>struct</b> <a href="#0x1_LibraVMConfig_GasSchedule">GasSchedule</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>instruction_schedule: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>native_schedule: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>gas_constants: <a href="#0x1_LibraVMConfig_GasConstants">LibraVMConfig::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_GasConstants"></a>

## Struct `GasConstants`



<pre><code><b>struct</b> <a href="#0x1_LibraVMConfig_GasConstants">GasConstants</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>global_memory_per_byte_cost: u64</code>
</dt>
<dd>
 The cost per-byte written to global storage.
</dd>
<dt>

<code>global_memory_per_byte_write_cost: u64</code>
</dt>
<dd>
 The cost per-byte written to storage.
</dd>
<dt>

<code>min_transaction_gas_units: u64</code>
</dt>
<dd>
 We charge one unit of gas per-byte for the first 600 bytes
</dd>
<dt>

<code>large_transaction_cutoff: u64</code>
</dt>
<dd>
 Any transaction over this size will be charged
<code>INTRINSIC_GAS_PER_BYTE</code> per byte
</dd>
<dt>

<code>instrinsic_gas_per_byte: u64</code>
</dt>
<dd>
 The units of gas that should be charged per byte for every transaction.
</dd>
<dt>

<code>maximum_number_of_gas_units: u64</code>
</dt>
<dd>
 1 nanosecond should equal one unit of computational gas. We bound the maximum
 computational time of any given transaction at 10 milliseconds. We want this number and
 <code>MAX_PRICE_PER_GAS_UNIT</code> to always satisfy the inequality that
         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
</dd>
<dt>

<code>min_price_per_gas_unit: u64</code>
</dt>
<dd>
 The minimum gas price that a transaction can be submitted with.
</dd>
<dt>

<code>max_price_per_gas_unit: u64</code>
</dt>
<dd>
 The maximum gas unit price that a transaction can be submitted with.
</dd>
<dt>

<code>max_transaction_size_in_bytes: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_initialize">initialize</a>(config_account: &signer, association_root_account: &signer, create_config_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_CreateOnChainConfig">LibraConfig::CreateOnChainConfig</a>&gt;, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_initialize">initialize</a>(
    config_account: &signer,
    association_root_account: &signer,
    create_config_capability: &Capability&lt;CreateOnChainConfig&gt;,
    publishing_option: vector&lt;u8&gt;,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
) {
    <b>let</b> gas_constants = <a href="#0x1_LibraVMConfig_GasConstants">GasConstants</a> {
        global_memory_per_byte_cost: 8,
        global_memory_per_byte_write_cost: 8,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: 600,
        instrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units: 2000000,
        min_price_per_gas_unit: 0,
        max_price_per_gas_unit: 10000,
        max_transaction_size_in_bytes: 4096,
    };


    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_with_delegate">LibraConfig::publish_new_config_with_delegate</a>&lt;<a href="#0x1_LibraVMConfig">LibraVMConfig</a>&gt;(
        config_account,
        create_config_capability,
        <a href="#0x1_LibraVMConfig">LibraVMConfig</a> {
            publishing_option,
            gas_schedule: <a href="#0x1_LibraVMConfig_GasSchedule">GasSchedule</a> {
                instruction_schedule,
                native_schedule,
                gas_constants,
            }
        },
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association_root_account),
    );
    <a href="LibraConfig.md#0x1_LibraConfig_claim_delegated_modify_config">LibraConfig::claim_delegated_modify_config</a>&lt;<a href="#0x1_LibraVMConfig">LibraVMConfig</a>&gt;(association_root_account, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account));
}
</code></pre>



</details>

<a name="0x1_LibraVMConfig_set_publishing_option"></a>

## Function `set_publishing_option`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_set_publishing_option">set_publishing_option</a>(account: &signer, publishing_option: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_set_publishing_option">set_publishing_option</a>(account: &signer, publishing_option: vector&lt;u8&gt;) {
    <b>let</b> current_config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraVMConfig">LibraVMConfig</a>&gt;();
    current_config.publishing_option = publishing_option;
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x1_LibraVMConfig">LibraVMConfig</a>&gt;(account, current_config);
}
</code></pre>



</details>
