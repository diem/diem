
<a name="0x0_VASP"></a>

# Module `0x0::VASP`

### Table of Contents

-  [Struct `ParentVASP`](#0x0_VASP_ParentVASP)
-  [Struct `ChildVASP`](#0x0_VASP_ChildVASP)
-  [Struct `RegistrationCapability`](#0x0_VASP_RegistrationCapability)
-  [Function `initialize`](#0x0_VASP_initialize)
-  [Function `recertify_vasp`](#0x0_VASP_recertify_vasp)
-  [Function `decertify_vasp`](#0x0_VASP_decertify_vasp)
-  [Function `delist_vasp`](#0x0_VASP_delist_vasp)
-  [Function `register_vasp`](#0x0_VASP_register_vasp)
-  [Function `create_parent_vasp_credential`](#0x0_VASP_create_parent_vasp_credential)
-  [Function `create_child_vasp`](#0x0_VASP_create_child_vasp)
-  [Function `child_parent_address`](#0x0_VASP_child_parent_address)
-  [Function `is_parent_vasp`](#0x0_VASP_is_parent_vasp)
-  [Function `human_name`](#0x0_VASP_human_name)
-  [Function `base_url`](#0x0_VASP_base_url)
-  [Function `compliance_public_key`](#0x0_VASP_compliance_public_key)
-  [Function `rotate_compliance_public_key`](#0x0_VASP_rotate_compliance_public_key)
-  [Function `expiration_date`](#0x0_VASP_expiration_date)
-  [Function `parent_credential_expired`](#0x0_VASP_parent_credential_expired)
-  [Function `singleton_addr`](#0x0_VASP_singleton_addr)
-  [Function `cert_lifetime`](#0x0_VASP_cert_lifetime)



<a name="0x0_VASP_ParentVASP"></a>

## Struct `ParentVASP`



<pre><code><b>struct</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>human_name: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>base_url: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>expiration_date: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>compliance_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASP_ChildVASP"></a>

## Struct `ChildVASP`



<pre><code><b>struct</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>parent_vasp_addr: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASP_RegistrationCapability"></a>

## Struct `RegistrationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="vasp_registry.md#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistry::VASPRegistrationCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASP_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_initialize">initialize</a>(config_account: &signer) {
    <b>let</b> cap = <a href="vasp_registry.md#0x0_VASPRegistry_initialize">VASPRegistry::initialize</a>(config_account);
    move_to(config_account, <a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a>{cap});
}
</code></pre>



</details>

<a name="0x0_VASP_recertify_vasp"></a>

## Function `recertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>) {
    parent_vasp.expiration_date = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>();
}
</code></pre>



</details>

<a name="0x0_VASP_decertify_vasp"></a>

## Function `decertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp_addr: address, parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp_addr: address, parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>)
<b>acquires</b> <a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a> {
    // Expire the parent credential.
    parent_vasp.expiration_date = 0;
    <a href="#0x0_VASP_delist_vasp">delist_vasp</a>(parent_vasp_addr)
}
</code></pre>



</details>

<a name="0x0_VASP_delist_vasp"></a>

## Function `delist_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_delist_vasp">delist_vasp</a>(parent_vasp_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_delist_vasp">delist_vasp</a>(parent_vasp_addr: address)
<b>acquires</b> <a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <b>let</b> cap = borrow_global&lt;<a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a>&gt;(<a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>());
    <a href="vasp_registry.md#0x0_VASPRegistry_remove_vasp">VASPRegistry::remove_vasp</a>(parent_vasp_addr, &cap.cap);
}
</code></pre>



</details>

<a name="0x0_VASP_register_vasp"></a>

## Function `register_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_register_vasp">register_vasp</a>(parent_vasp_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_register_vasp">register_vasp</a>(parent_vasp_addr: address)
<b>acquires</b> <a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <b>let</b> cap = borrow_global&lt;<a href="#0x0_VASP_RegistrationCapability">RegistrationCapability</a>&gt;(<a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>());
    <a href="vasp_registry.md#0x0_VASPRegistry_add_vasp">VASPRegistry::add_vasp</a>(parent_vasp_addr, &cap.cap);
}
</code></pre>



</details>

<a name="0x0_VASP_create_parent_vasp_credential"></a>

## Function `create_parent_vasp_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_parent_vasp_credential">create_parent_vasp_credential</a>(human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;): <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_parent_vasp_credential">create_parent_vasp_credential</a>(
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;
): <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    // NOTE: Only callable in testnet
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>(), 10041);
    <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
       // For testnet, so it should never expire. So set <b>to</b> u64::MAX
       expiration_date: 18446744073709551615,
       human_name,
       base_url,
       compliance_public_key,
    }
}
</code></pre>



</details>

<a name="0x0_VASP_create_child_vasp"></a>

## Function `create_child_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_child_vasp">create_child_vasp</a>(sender: &signer): <a href="#0x0_VASP_ChildVASP">VASP::ChildVASP</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_child_vasp">create_child_vasp</a>(sender: &signer): <a href="#0x0_VASP_ChildVASP">ChildVASP</a> {
    <a href="#0x0_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr: <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(sender) }
}
</code></pre>



</details>

<a name="0x0_VASP_child_parent_address"></a>

## Function `child_parent_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_child_parent_address">child_parent_address</a>(child: &<a href="#0x0_VASP_ChildVASP">VASP::ChildVASP</a>): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_child_parent_address">child_parent_address</a>(child: &<a href="#0x0_VASP_ChildVASP">ChildVASP</a>): address {
    child.parent_vasp_addr
}
</code></pre>



</details>

<a name="0x0_VASP_is_parent_vasp"></a>

## Function `is_parent_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent_vasp">is_parent_vasp</a>(child_vasp: &<a href="#0x0_VASP_ChildVASP">VASP::ChildVASP</a>, sender: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent_vasp">is_parent_vasp</a>(child_vasp: &<a href="#0x0_VASP_ChildVASP">ChildVASP</a>, sender: &signer): bool {
    <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(sender) == child_vasp.parent_vasp_addr
}
</code></pre>



</details>

<a name="0x0_VASP_human_name"></a>

## Function `human_name`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">ParentVASP</a>): vector&lt;u8&gt; {
    *&parent_vasp.human_name
}
</code></pre>



</details>

<a name="0x0_VASP_base_url"></a>

## Function `base_url`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">ParentVASP</a>): vector&lt;u8&gt; {
    *&parent_vasp.base_url
}
</code></pre>



</details>

<a name="0x0_VASP_compliance_public_key"></a>

## Function `compliance_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">ParentVASP</a>): vector&lt;u8&gt; {
    *&parent_vasp.compliance_public_key
}
</code></pre>



</details>

<a name="0x0_VASP_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`

Rotate the compliance public key for
<code>parent_vasp</code> to
<code>new_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>, new_key: vector&lt;u8&gt;) {
    Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&new_key) == 32, 7004);
    parent_vasp.compliance_public_key = new_key;
}
</code></pre>



</details>

<a name="0x0_VASP_expiration_date"></a>

## Function `expiration_date`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(parent_vasp: &<a href="#0x0_VASP_ParentVASP">ParentVASP</a>): u64 {
    parent_vasp.expiration_date
}
</code></pre>



</details>

<a name="0x0_VASP_parent_credential_expired"></a>

## Function `parent_credential_expired`



<pre><code><b>fun</b> <a href="#0x0_VASP_parent_credential_expired">parent_credential_expired</a>(parent_credential: &<a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_VASP_parent_credential_expired">parent_credential_expired</a>(parent_credential: &<a href="#0x0_VASP_ParentVASP">ParentVASP</a>): bool {
    parent_credential.<a href="#0x0_VASP_expiration_date">expiration_date</a> &lt; <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>()
}
</code></pre>



</details>

<a name="0x0_VASP_singleton_addr"></a>

## Function `singleton_addr`



<pre><code><b>fun</b> <a href="#0x0_VASP_singleton_addr">singleton_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_VASP_singleton_addr">singleton_addr</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x0_VASP_cert_lifetime"></a>

## Function `cert_lifetime`



<pre><code><b>fun</b> <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>(): u64 {
    31540000000000
}
</code></pre>



</details>
