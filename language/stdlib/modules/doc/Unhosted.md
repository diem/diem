
<a name="0x1_Unhosted"></a>

# Module `0x1::Unhosted`

### Table of Contents

-  [Struct `Unhosted`](#0x1_Unhosted_Unhosted)
-  [Function `publish_global_limits_definition`](#0x1_Unhosted_publish_global_limits_definition)
-  [Function `create`](#0x1_Unhosted_create)
-  [Function `window_length`](#0x1_Unhosted_window_length)



<a name="0x1_Unhosted_Unhosted"></a>

## Struct `Unhosted`



<pre><code><b>struct</b> <a href="#0x1_Unhosted">Unhosted</a>
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

<a name="0x1_Unhosted_publish_global_limits_definition"></a>

## Function `publish_global_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>(account: &signer, cap: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>(account: &signer, cap: &Capability&lt;TreasuryComplianceRole&gt;) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), 100042);
    // These are limits for testnet _only_.
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>(account);
    /*<a href="AccountLimits.md#0x1_AccountLimits_publish_limits_definition">AccountLimits::publish_limits_definition</a>(
        10000,
        10000,
        50000,
        31540000000000
    );*/
    <a href="AccountLimits.md#0x1_AccountLimits_certify_limits_definition">AccountLimits::certify_limits_definition</a>(cap, <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>());
}
</code></pre>



</details>

<a name="0x1_Unhosted_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Unhosted_create">create</a>(): <a href="#0x1_Unhosted_Unhosted">Unhosted::Unhosted</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Unhosted_create">create</a>(): <a href="#0x1_Unhosted">Unhosted</a> {
    <b>assert</b>(<a href="Testnet.md#0x1_Testnet_is_testnet">Testnet::is_testnet</a>(), 10041);
    <a href="#0x1_Unhosted">Unhosted</a> {  }
}
</code></pre>



</details>

<a name="0x1_Unhosted_window_length"></a>

## Function `window_length`



<pre><code><b>fun</b> <a href="#0x1_Unhosted_window_length">window_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Unhosted_window_length">window_length</a>(): u64 {
    // number of microseconds in a day
    86400000000
}
</code></pre>



</details>
