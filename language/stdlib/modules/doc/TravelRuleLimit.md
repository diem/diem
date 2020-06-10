
<a name="0x0_TravelRuleLimit"></a>

# Module `0x0::TravelRuleLimit`

### Table of Contents

-  [Struct `TravelRuleLimit`](#0x0_TravelRuleLimit_TravelRuleLimit)
-  [Struct `ModifyCapability`](#0x0_TravelRuleLimit_ModifyCapability)
-  [Function `initialize`](#0x0_TravelRuleLimit_initialize)
-  [Function `get_cur_microlibra_limit`](#0x0_TravelRuleLimit_get_cur_microlibra_limit)
-  [Function `set_microlibra_limit`](#0x0_TravelRuleLimit_set_microlibra_limit)



<a name="0x0_TravelRuleLimit_TravelRuleLimit"></a>

## Struct `TravelRuleLimit`



<pre><code><b>struct</b> <a href="#0x0_TravelRuleLimit">TravelRuleLimit</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>micro_libra_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TravelRuleLimit_ModifyCapability"></a>

## Struct `ModifyCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_TravelRuleLimit_ModifyCapability">ModifyCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraConfig.md#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x0_TravelRuleLimit_TravelRuleLimit">TravelRuleLimit::TravelRuleLimit</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TravelRuleLimit_initialize"></a>

## Function `initialize`

Travel rule limit set during genesis


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_initialize">initialize</a>(account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_initialize">initialize</a>(account: &signer, tc_account: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="LibraConfig.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(), 1);
    <b>let</b> cap = <a href="LibraConfig.md#0x0_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>&lt;<a href="#0x0_TravelRuleLimit">TravelRuleLimit</a>&gt;(
        account,
        <a href="#0x0_TravelRuleLimit">TravelRuleLimit</a> { micro_libra_limit: 1000000 },
    );
    move_to(tc_account, <a href="#0x0_TravelRuleLimit_ModifyCapability">ModifyCapability</a> { cap })
}
</code></pre>



</details>

<a name="0x0_TravelRuleLimit_get_cur_microlibra_limit"></a>

## Function `get_cur_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64 {
    <b>let</b> rule = <a href="LibraConfig.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_TravelRuleLimit">TravelRuleLimit</a>&gt;();
    rule.micro_libra_limit
}
</code></pre>



</details>

<a name="0x0_TravelRuleLimit_set_microlibra_limit"></a>

## Function `set_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_set_microlibra_limit">set_microlibra_limit</a>(tc_account: &signer, micro_libra_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TravelRuleLimit_set_microlibra_limit">set_microlibra_limit</a>(tc_account: &signer, micro_libra_limit: u64) <b>acquires</b> <a href="#0x0_TravelRuleLimit_ModifyCapability">ModifyCapability</a> {
    Transaction::assert(
        micro_libra_limit &gt; 1000,
        4
    );
    <b>let</b> tc_address = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(tc_account);
    Transaction::assert(tc_address == <a href="Association.md#0x0_Association_treasury_compliance_account">Association::treasury_compliance_account</a>(), 3);
    <b>let</b> modify_cap = &borrow_global&lt;<a href="#0x0_TravelRuleLimit_ModifyCapability">ModifyCapability</a>&gt;(tc_address).cap;
    // <b>let</b> modify_cap = borrow_global&lt;<a href="#0x0_TravelRuleLimit_ModifyCapability">ModifyCapability</a>&lt;<a href="#0x0_TravelRuleLimit_TravelRuleLimit">Self::TravelRuleLimit</a>&gt;&gt;(tc_address).cap;
    <a href="LibraConfig.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>&lt;<a href="#0x0_TravelRuleLimit">TravelRuleLimit</a>&gt;(
        modify_cap,
        <a href="#0x0_TravelRuleLimit">TravelRuleLimit</a> { micro_libra_limit },
    );
}
</code></pre>



</details>
