
<a name="0x1_DiemVMConfig"></a>

# Module `0x1::DiemVMConfig`

This module defines structs and methods to initialize VM configurations,
including different costs of running the VM.


-  [Resource `VMConfigChainMarker`](#0x1_DiemVMConfig_VMConfigChainMarker)
-  [Resource `DiemVMConfig`](#0x1_DiemVMConfig_DiemVMConfig)
-  [Struct `GasSchedule`](#0x1_DiemVMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_DiemVMConfig_GasConstants)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemVMConfig_initialize)
-  [Function `set_gas_constants`](#0x1_DiemVMConfig_set_gas_constants)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_DiemVMConfig_VMConfigChainMarker"></a>

## Resource `VMConfigChainMarker`

Marker to be stored under @CoreResources during genesis


<pre><code><b>struct</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_VMConfigChainMarker">VMConfigChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_DiemVMConfig_DiemVMConfig"></a>

## Resource `DiemVMConfig`

The struct to hold config data needed to operate the DiemVM.


<pre><code><b>struct</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>gas_schedule: <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasSchedule">DiemVMConfig::GasSchedule</a></code>
</dt>
<dd>
 Cost of running the VM.
</dd>
</dl>


</details>

<a name="0x1_DiemVMConfig_GasSchedule"></a>

## Struct `GasSchedule`

The gas schedule keeps two separate schedules for the gas:
* The instruction_schedule: This holds the gas for each bytecode instruction.
* The native_schedule: This holds the gas for used (per-byte operated over) for each native
function.
A couple notes:
1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
still needs to remain the same; once a slot in the table is taken by an instruction, that is its
slot for the rest of time (since that instruction could already exist in a module on-chain).
2. The initialization of the module will publish the instruction table to the diem root account
address, and will preload the vector with the gas schedule for instructions. The VM will then
load this into memory at the startup of each block.


<pre><code><b>struct</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasSchedule">GasSchedule</a> <b>has</b> <b>copy</b>, drop, store
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
<code>gas_constants: <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasConstants">DiemVMConfig::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DiemVMConfig_GasConstants"></a>

## Struct `GasConstants`



<pre><code><b>struct</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasConstants">GasConstants</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>global_memory_per_byte_cost: u64</code>
</dt>
<dd>
 The cost per-byte read from global storage.
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
 The flat minimum amount of gas required for any transaction.
 Charged at the start of execution.
</dd>
<dt>
<code>large_transaction_cutoff: u64</code>
</dt>
<dd>
 Any transaction over this size will be charged an additional amount per byte.
</dd>
<dt>
<code>intrinsic_gas_per_byte: u64</code>
</dt>
<dd>
 The units of gas to be charged per byte over the <code>large_transaction_cutoff</code> in addition to
 <code>min_transaction_gas_units</code> for transactions whose size exceeds <code>large_transaction_cutoff</code>.
</dd>
<dt>
<code>maximum_number_of_gas_units: u64</code>
</dt>
<dd>
 ~5 microseconds should equal one unit of computational gas. We bound the maximum
 computational time of any given transaction at roughly 20 seconds. We want this number and
 <code>MAX_PRICE_PER_GAS_UNIT</code> to always satisfy the inequality that
 MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
 NB: The bound is set quite high since custom scripts aren't allowed except from predefined
 and vetted senders.
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemVMConfig_ECHAIN_MARKER"></a>

Error with chain marker


<pre><code><b>const</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemVMConfig_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_ECONFIG">ECONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemVMConfig_EGAS_CONSTANT_INCONSISTENCY"></a>

The provided gas constants were inconsistent.


<pre><code><b>const</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_EGAS_CONSTANT_INCONSISTENCY">EGAS_CONSTANT_INCONSISTENCY</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemVMConfig_initialize"></a>

## Function `initialize`

Initialize the table under the diem root account


<pre><code><b>public</b> <b>fun</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_initialize">initialize</a>&lt;T&gt;(account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_initialize">initialize</a>&lt;T&gt;(
    account: &signer,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();

    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemVMConfig.md#0x1_DiemVMConfig_VMConfigChainMarker">VMConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a>&gt;(@CoreResources),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_ECONFIG">ECONFIG</a>)
    );

    <b>move_to</b>(account, <a href="DiemVMConfig.md#0x1_DiemVMConfig_VMConfigChainMarker">VMConfigChainMarker</a>&lt;T&gt;{});

    <b>let</b> gas_constants = <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasConstants">GasConstants</a> {
        global_memory_per_byte_cost: 4,
        global_memory_per_byte_write_cost: 9,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: 600,
        intrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units: 4000000,
        min_price_per_gas_unit: 0,
        max_price_per_gas_unit: 10000,
        max_transaction_size_in_bytes: 4096,
        gas_unit_scaling_factor: 1000,
        default_account_size: 800,
    };

    <b>move_to</b>(
        account,
        <a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a> {
            gas_schedule: <a href="DiemVMConfig.md#0x1_DiemVMConfig_GasSchedule">GasSchedule</a> {
                instruction_schedule,
                native_schedule,
                gas_constants,
            }
        },
    );
}
</code></pre>



</details>

<a name="0x1_DiemVMConfig_set_gas_constants"></a>

## Function `set_gas_constants`



<pre><code><b>public</b> <b>fun</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_set_gas_constants">set_gas_constants</a>&lt;T&gt;(global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, intrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig_set_gas_constants">set_gas_constants</a>&lt;T&gt;(
    global_memory_per_byte_cost: u64,
    global_memory_per_byte_write_cost: u64,
    min_transaction_gas_units: u64,
    large_transaction_cutoff: u64,
    intrinsic_gas_per_byte: u64,
    maximum_number_of_gas_units: u64,
    min_price_per_gas_unit: u64,
    max_price_per_gas_unit: u64,
    max_transaction_size_in_bytes: u64,
    gas_unit_scaling_factor: u64,
    default_account_size: u64,
    _cap: &Cap&lt;T&gt;
) <b>acquires</b> <a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();

    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemVMConfig.md#0x1_DiemVMConfig_VMConfigChainMarker">VMConfigChainMarker</a>&lt;T&gt;&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_ECHAIN_MARKER">ECHAIN_MARKER</a>));

    <b>assert</b>!(
        min_price_per_gas_unit &lt;= max_price_per_gas_unit,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_EGAS_CONSTANT_INCONSISTENCY">EGAS_CONSTANT_INCONSISTENCY</a>)
    );
    <b>assert</b>!(
        min_transaction_gas_units &lt;= maximum_number_of_gas_units,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_EGAS_CONSTANT_INCONSISTENCY">EGAS_CONSTANT_INCONSISTENCY</a>)
    );

    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a>&gt;(@CoreResources), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemVMConfig.md#0x1_DiemVMConfig_ECONFIG">ECONFIG</a>));

    <b>let</b> gas_constants = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a>&gt;(@CoreResources).gas_schedule.gas_constants;

    gas_constants.global_memory_per_byte_cost       = global_memory_per_byte_cost;
    gas_constants.global_memory_per_byte_write_cost = global_memory_per_byte_write_cost;
    gas_constants.min_transaction_gas_units         = min_transaction_gas_units;
    gas_constants.large_transaction_cutoff          = large_transaction_cutoff;
    gas_constants.intrinsic_gas_per_byte            = intrinsic_gas_per_byte;
    gas_constants.maximum_number_of_gas_units       = maximum_number_of_gas_units;
    gas_constants.min_price_per_gas_unit            = min_price_per_gas_unit;
    gas_constants.max_price_per_gas_unit            = max_price_per_gas_unit;
    gas_constants.max_transaction_size_in_bytes     = max_transaction_size_in_bytes;
    gas_constants.gas_unit_scaling_factor           = gas_unit_scaling_factor;
    gas_constants.default_account_size              = default_account_size;

    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>();
}
</code></pre>



</details>
