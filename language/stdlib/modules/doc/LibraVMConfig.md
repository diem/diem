
<a name="0x1_LibraVMConfig"></a>

# Module `0x1::LibraVMConfig`

### Table of Contents

-  [Struct `LibraVMConfig`](#0x1_LibraVMConfig_LibraVMConfig)
-  [Struct `GasSchedule`](#0x1_LibraVMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_LibraVMConfig_GasConstants)
-  [Function `initialize`](#0x1_LibraVMConfig_initialize)
-  [Function `set_publishing_option`](#0x1_LibraVMConfig_set_publishing_option)
-  [Specification](#0x1_LibraVMConfig_Specification)
    -  [Function `initialize`](#0x1_LibraVMConfig_Specification_initialize)



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
<dt>

<code>gas_unit_scaling_factor: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>default_account_size: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_initialize">initialize</a>(lr_account: &signer, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_initialize">initialize</a>(
    lr_account: &signer,
    publishing_option: vector&lt;u8&gt;,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
) {
    <b>let</b> gas_constants = <a href="#0x1_LibraVMConfig_GasConstants">GasConstants</a> {
        global_memory_per_byte_cost: 4,
        global_memory_per_byte_write_cost: 9,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: 600,
        instrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units: 4000000,
        min_price_per_gas_unit: 0,
        max_price_per_gas_unit: 10000,
        max_transaction_size_in_bytes: 4096,
        gas_unit_scaling_factor: 1000,
        default_account_size: 800,
    };

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="#0x1_LibraVMConfig">LibraVMConfig</a> {
            publishing_option,
            gas_schedule: <a href="#0x1_LibraVMConfig_GasSchedule">GasSchedule</a> {
                instruction_schedule,
                native_schedule,
                gas_constants,
            }
        },
    );
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

<a name="0x1_LibraVMConfig_Specification"></a>

## Specification


<a name="0x1_LibraVMConfig_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraVMConfig_initialize">initialize</a>(lr_account: &signer, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma aborts_if_is_partial = <b>true</b>;
</code></pre>



The permission "UpdateVMConfig" is granted to LibraRoot [B21].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account} <b>to</b> initialize;
</code></pre>
