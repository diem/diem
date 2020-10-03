
<a name="0x1_LibraVMConfig"></a>

# Module `0x1::LibraVMConfig`



-  [Struct `LibraVMConfig`](#0x1_LibraVMConfig_LibraVMConfig)
-  [Struct `GasSchedule`](#0x1_LibraVMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_LibraVMConfig_GasConstants)
-  [Function `initialize`](#0x1_LibraVMConfig_initialize)
-  [Module Specification](#@Module_Specification_0)


<pre><code><b>use</b> <a href="LibraConfig.md#0x1_LibraConfig">0x1::LibraConfig</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
</code></pre>



<a name="0x1_LibraVMConfig_LibraVMConfig"></a>

## Struct `LibraVMConfig`



<pre><code><b>struct</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>gas_schedule: <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasSchedule">LibraVMConfig::GasSchedule</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_GasSchedule"></a>

## Struct `GasSchedule`



<pre><code><b>struct</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasSchedule">GasSchedule</a>
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
<code>gas_constants: <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasConstants">LibraVMConfig::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraVMConfig_GasConstants"></a>

## Struct `GasConstants`



<pre><code><b>struct</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasConstants">GasConstants</a>
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
 The units of gas that to be charged per byte over the <code>large_transaction_cutoff</code> in addition to
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

<a name="0x1_LibraVMConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_initialize">initialize</a>(lr_account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_initialize">initialize</a>(
    lr_account: &signer,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();

    // The permission "UpdateVMConfig" is granted <b>to</b> LibraRoot [[H10]][PERMISSION].
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> gas_constants = <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasConstants">GasConstants</a> {
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

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a> {
            gas_schedule: <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasSchedule">GasSchedule</a> {
                instruction_schedule,
                native_schedule,
                gas_constants,
            }
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_LibraVMConfig_gas_constants$1"></a>


<pre><code><b>let</b> gas_constants = <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasConstants">GasConstants</a> {
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
</code></pre>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">LibraConfig::PublishNewConfigAbortsIf</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">LibraConfig::PublishNewConfigEnsures</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt; {
    payload: <a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a> {
        gas_schedule: <a href="LibraVMConfig.md#0x1_LibraVMConfig_GasSchedule">GasSchedule</a> {
            instruction_schedule,
            native_schedule,
            gas_constants,
        }
    }};
</code></pre>


Currently, no one can update LibraVMConfig [[H10]][PERMISSION]


<a name="0x1_LibraVMConfig_LibraVMConfigRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_LibraVMConfigRemainsSame">LibraVMConfigRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig_LibraVMConfigRemainsSame">LibraVMConfigRemainsSame</a> <b>to</b> *;
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraVMConfig.md#0x1_LibraVMConfig">LibraVMConfig</a>&gt;();
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
