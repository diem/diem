
<a name="0x1_DualAttestationLimit"></a>

# Module `0x1::DualAttestationLimit`

### Table of Contents

-  [Resource `UpdateDualAttestationThreshold`](#0x1_DualAttestationLimit_UpdateDualAttestationThreshold)
-  [Struct `DualAttestationLimit`](#0x1_DualAttestationLimit_DualAttestationLimit)
-  [Resource `ModifyLimitCapability`](#0x1_DualAttestationLimit_ModifyLimitCapability)
-  [Function `initialize`](#0x1_DualAttestationLimit_initialize)
-  [Function `get_cur_microlibra_limit`](#0x1_DualAttestationLimit_get_cur_microlibra_limit)
-  [Function `set_microlibra_limit`](#0x1_DualAttestationLimit_set_microlibra_limit)
-  [Specification](#0x1_DualAttestationLimit_Specification)



<a name="0x1_DualAttestationLimit_UpdateDualAttestationThreshold"></a>

## Resource `UpdateDualAttestationThreshold`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DualAttestationLimit_UpdateDualAttestationThreshold">UpdateDualAttestationThreshold</a>
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

<a name="0x1_DualAttestationLimit_DualAttestationLimit"></a>

## Struct `DualAttestationLimit`



<pre><code><b>struct</b> <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>micro_lbr_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DualAttestationLimit_ModifyLimitCapability"></a>

## Resource `ModifyLimitCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x1_DualAttestationLimit_DualAttestationLimit">DualAttestationLimit::DualAttestationLimit</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DualAttestationLimit_initialize"></a>

## Function `initialize`

Travel rule limit set during genesis


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    <b>let</b> cap = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;(
        lr_account,
        <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a> { micro_lbr_limit: INITIAL_DUAL_ATTESTATION_THRESHOLD * <a href="Libra.md#0x1_Libra_scaling_factor">Libra::scaling_factor</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;() },
    );
    move_to(tc_account, <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a> { cap })
}
</code></pre>



</details>

<a name="0x1_DualAttestationLimit_get_cur_microlibra_limit"></a>

## Function `get_cur_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64 {
    <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;().micro_lbr_limit
}
</code></pre>



</details>

<a name="0x1_DualAttestationLimit_set_microlibra_limit"></a>

## Function `set_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_set_microlibra_limit">set_microlibra_limit</a>(tc_account: &signer, micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_set_microlibra_limit">set_microlibra_limit</a>(
    tc_account: &signer,
    micro_lbr_limit: u64
) <b>acquires</b> <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_update_dual_attestation_threshold_privilege">Roles::has_update_dual_attestation_threshold_privilege</a>(tc_account), ECANNOT_UPDATE_THRESHOLD);
    <b>assert</b>(micro_lbr_limit &gt;= 1000, ETHRESHOLD_TOO_LOW);
    <b>let</b> tc_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account);
    <b>let</b> modify_cap = &borrow_global&lt;<a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a>&gt;(tc_address).cap;
    <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;(
        modify_cap,
        <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a> { micro_lbr_limit },
    );
}
</code></pre>



</details>

<a name="0x1_DualAttestationLimit_Specification"></a>

## Specification



<pre><code>pragma verify = <b>true</b>;
</code></pre>


Helper function to determine whether the DualAttestationLimit is published.


<a name="0x1_DualAttestationLimit_spec_is_published"></a>


<pre><code><b>define</b> <a href="#0x1_DualAttestationLimit_spec_is_published">spec_is_published</a>(): bool {
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;()
}
</code></pre>


Mirrors
<code><a href="#0x1_DualAttestationLimit_get_cur_microlibra_limit">Self::get_cur_microlibra_limit</a></code>.


<a name="0x1_DualAttestationLimit_spec_get_cur_microlibra_limit"></a>


<pre><code><b>define</b> <a href="#0x1_DualAttestationLimit_spec_get_cur_microlibra_limit">spec_get_cur_microlibra_limit</a>(): u64 {
    <a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;().micro_lbr_limit
}
</code></pre>


After genesis, the DualAttestationLimit is always published.


<pre><code><b>invariant</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>() ==&gt; <a href="#0x1_DualAttestationLimit_spec_is_published">spec_is_published</a>();
</code></pre>
