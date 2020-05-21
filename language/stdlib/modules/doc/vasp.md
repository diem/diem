
<a name="0x0_VASP"></a>

# Module `0x0::VASP`

### Table of Contents

-  [Struct `RootVASP`](#0x0_VASP_RootVASP)
-  [Struct `ChildVASP`](#0x0_VASP_ChildVASP)
-  [Struct `CreationPrivilege`](#0x0_VASP_CreationPrivilege)
-  [Function `initialize`](#0x0_VASP_initialize)
-  [Function `recertify_vasp`](#0x0_VASP_recertify_vasp)
-  [Function `grant_vasp`](#0x0_VASP_grant_vasp)
-  [Function `decertify_vasp`](#0x0_VASP_decertify_vasp)
-  [Function `rotate_compliance_public_key`](#0x0_VASP_rotate_compliance_public_key)
-  [Function `create_root_vasp_credential`](#0x0_VASP_create_root_vasp_credential)
-  [Function `apply_for_vasp_root_credential`](#0x0_VASP_apply_for_vasp_root_credential)
-  [Function `allow_child_accounts`](#0x0_VASP_allow_child_accounts)
-  [Function `apply_for_parent_capability`](#0x0_VASP_apply_for_parent_capability)
-  [Function `grant_parent_capability`](#0x0_VASP_grant_parent_capability)
-  [Function `remove_parent_capability`](#0x0_VASP_remove_parent_capability)
-  [Function `grant_child_account`](#0x0_VASP_grant_child_account)
-  [Function `decertify_child_account`](#0x0_VASP_decertify_child_account)
-  [Function `recertify_child_account`](#0x0_VASP_recertify_child_account)
-  [Function `apply_for_child_vasp_credential`](#0x0_VASP_apply_for_child_vasp_credential)
-  [Function `is_root_vasp`](#0x0_VASP_is_root_vasp)
-  [Function `is_child_vasp`](#0x0_VASP_is_child_vasp)
-  [Function `root_vasp_address`](#0x0_VASP_root_vasp_address)
-  [Function `is_root_child_vasp`](#0x0_VASP_is_root_child_vasp)
-  [Function `is_parent_vasp`](#0x0_VASP_is_parent_vasp)
-  [Function `is_vasp`](#0x0_VASP_is_vasp)
-  [Function `allows_child_accounts`](#0x0_VASP_allows_child_accounts)
-  [Function `human_name`](#0x0_VASP_human_name)
-  [Function `base_url`](#0x0_VASP_base_url)
-  [Function `compliance_public_key`](#0x0_VASP_compliance_public_key)
-  [Function `expiration_date`](#0x0_VASP_expiration_date)
-  [Function `root_credential_expired`](#0x0_VASP_root_credential_expired)
-  [Function `assert_sender_is_assoc_vasp_privileged`](#0x0_VASP_assert_sender_is_assoc_vasp_privileged)
-  [Function `singleton_addr`](#0x0_VASP_singleton_addr)
-  [Function `cert_lifetime`](#0x0_VASP_cert_lifetime)



<a name="0x0_VASP_RootVASP"></a>

## Struct `RootVASP`



<pre><code><b>struct</b> <a href="#0x0_VASP_RootVASP">RootVASP</a>
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

<code>is_certified: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASP_CreationPrivilege"></a>

## Struct `CreationPrivilege`



<pre><code><b>struct</b> <a href="#0x0_VASP_CreationPrivilege">CreationPrivilege</a>
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

<a name="0x0_VASP_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_initialize">initialize</a>() {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <b>let</b> sender = Transaction::sender();
    Transaction::assert(sender == <a href="#0x0_VASP_singleton_addr">singleton_addr</a>(), 7000);
    <a href="account_type.md#0x0_AccountType_register">AccountType::register</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;();
    <a href="account_type.md#0x0_AccountType_register">AccountType::register</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;();
    // Now publish and certify this account <b>to</b> allow granting of root
    // <a href="#0x0_VASP">VASP</a> accounts. We can perform all of these operations at once
    // since the <a href="#0x0_VASP_singleton_addr">singleton_addr</a>() == sender, and <a href="#0x0_VASP_singleton_addr">singleton_addr</a>() must
    // be an association address.
    <a href="account_type.md#0x0_AccountType_apply_for_granting_capability">AccountType::apply_for_granting_capability</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;();
    <a href="account_type.md#0x0_AccountType_certify_granting_capability">AccountType::certify_granting_capability</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(sender);
    // Apply and certify that this account can transition <a href="empty.md#0x0_Empty_T">Empty::T</a> =&gt; <a href="#0x0_VASP_RootVASP">RootVASP</a>
    <a href="account_type.md#0x0_AccountType_apply_for_transition_capability">AccountType::apply_for_transition_capability</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(sender);
    <a href="account_type.md#0x0_AccountType_grant_transition_capability">AccountType::grant_transition_capability</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(sender);
}
</code></pre>



</details>

<a name="0x0_VASP_recertify_vasp"></a>

## Function `recertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(addr: address) {
    // Verify that the sender is a correctly privileged association account
    <a href="#0x0_VASP_assert_sender_is_assoc_vasp_privileged">assert_sender_is_assoc_vasp_privileged</a>();
    // Verify that the account in question is still a valid root <a href="#0x0_VASP">VASP</a> account
    // But, it's cert might have expired. That's why we don't <b>use</b>
    // `is_root_vasp` here, but instead make sure `addr` is a <a href="#0x0_VASP_RootVASP">RootVASP</a>
    // account type.
    Transaction::assert(<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr), 7001);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr);
    root_vasp.expiration_date = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>();
    // The sending account must have a TransitionCapability&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;.
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr, root_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_grant_vasp"></a>

## Function `grant_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_vasp">grant_vasp</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_vasp">grant_vasp</a>(addr: address) {
    <a href="#0x0_VASP_assert_sender_is_assoc_vasp_privileged">assert_sender_is_assoc_vasp_privileged</a>();
    // The sending account must have a TransitionCapability&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt; capability
    <a href="account_type.md#0x0_AccountType_transition">AccountType::transition</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr);
    <a href="account_type.md#0x0_AccountType_certify_granting_capability">AccountType::certify_granting_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_VASP_decertify_vasp"></a>

## Function `decertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(addr: address) {
    <a href="#0x0_VASP_assert_sender_is_assoc_vasp_privileged">assert_sender_is_assoc_vasp_privileged</a>();
    Transaction::assert(<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(addr), 7001);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr);
    // Expire the root credential.
    root_vasp.expiration_date = 0;
    // Updat the root vasp metadata for the account with the new root vasp
    // credential.
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr, root_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(root_vasp_addr: address, new_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(root_vasp_addr: address, new_public_key: vector&lt;u8&gt;) {
    Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&new_public_key) == 32, 7004);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr);
    root_vasp.compliance_public_key = new_public_key;
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr, root_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_create_root_vasp_credential"></a>

## Function `create_root_vasp_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_root_vasp_credential">create_root_vasp_credential</a>(human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;): <a href="#0x0_VASP_RootVASP">VASP::RootVASP</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_create_root_vasp_credential">create_root_vasp_credential</a>(
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;
): <a href="#0x0_VASP_RootVASP">RootVASP</a> {
    // NOTE: Only callable in testnet
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>(), 10041);
    <a href="#0x0_VASP_RootVASP">RootVASP</a> {
       // For testnet, so it should never expire. So set <b>to</b> u64::MAX
       expiration_date: 18446744073709551615,
       human_name,
       base_url,
       compliance_public_key,
    }
}
</code></pre>



</details>

<a name="0x0_VASP_apply_for_vasp_root_credential"></a>

## Function `apply_for_vasp_root_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_vasp_root_credential">apply_for_vasp_root_credential</a>(human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_vasp_root_credential">apply_for_vasp_root_credential</a>(
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;
) {
    // Sanity check for key validity
    Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&compliance_public_key) == 32, 7004);
    <a href="account_type.md#0x0_AccountType_apply_for">AccountType::apply_for</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(<a href="#0x0_VASP_RootVASP">RootVASP</a> {
        expiration_date: <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>(),
        human_name,
        base_url,
        compliance_public_key,
    }, <a href="#0x0_VASP_singleton_addr">singleton_addr</a>());
    <a href="account_type.md#0x0_AccountType_apply_for_granting_capability">AccountType::apply_for_granting_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;();
}
</code></pre>



</details>

<a name="0x0_VASP_allow_child_accounts"></a>

## Function `allow_child_accounts`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_allow_child_accounts">allow_child_accounts</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_allow_child_accounts">allow_child_accounts</a>() {
    <b>let</b> sender = Transaction::sender();
    <a href="account_type.md#0x0_AccountType_apply_for_transition_capability">AccountType::apply_for_transition_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(sender);
    <a href="account_type.md#0x0_AccountType_grant_transition_capability">AccountType::grant_transition_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(sender);
}
</code></pre>



</details>

<a name="0x0_VASP_apply_for_parent_capability"></a>

## Function `apply_for_parent_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_parent_capability">apply_for_parent_capability</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_parent_capability">apply_for_parent_capability</a>() {
    <b>let</b> sender = Transaction::sender();
    Transaction::assert(<a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(sender), 7002);
    <b>let</b> root_vasp_addr = <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(sender);
    // Apply for the ability <b>to</b> transition <a href="empty.md#0x0_Empty_T">Empty::T</a> =&gt; <a href="#0x0_VASP_ChildVASP">VASP::ChildVASP</a>
    // accounts with the root authority address being at `root_vasp_addr`
    <a href="account_type.md#0x0_AccountType_apply_for_transition_capability">AccountType::apply_for_transition_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(root_vasp_addr);
}
</code></pre>



</details>

<a name="0x0_VASP_grant_parent_capability"></a>

## Function `grant_parent_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_parent_capability">grant_parent_capability</a>(for_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_parent_capability">grant_parent_capability</a>(for_address: address) {
    <a href="account_type.md#0x0_AccountType_grant_transition_capability">AccountType::grant_transition_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(for_address);
}
</code></pre>



</details>

<a name="0x0_VASP_remove_parent_capability"></a>

## Function `remove_parent_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_remove_parent_capability">remove_parent_capability</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_remove_parent_capability">remove_parent_capability</a>(addr: address) {
    <a href="account_type.md#0x0_AccountType_remove_transition_capability">AccountType::remove_transition_capability</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_VASP_grant_child_account"></a>

## Function `grant_child_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_child_account">grant_child_account</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_grant_child_account">grant_child_account</a>(addr: address) {
    <b>let</b> root_account_addr = <a href="account_type.md#0x0_AccountType_transition_cap_root_addr">AccountType::transition_cap_root_addr</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(Transaction::sender());
    Transaction::assert(<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_account_addr), 7001);
    // Transition the child account: <a href="empty.md#0x0_Empty_T">Empty::T</a> =&gt; <a href="#0x0_VASP_ChildVASP">VASP::ChildVASP</a>.
    // The <a href="#0x0_VASP_ChildVASP">ChildVASP</a> account type must be published under the child, but not yet certified
    <a href="account_type.md#0x0_AccountType_transition">AccountType::transition</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
    <b>let</b> child_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
    // Now mark this account <b>as</b> certified
    child_vasp.is_certified = <b>true</b>;
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr, child_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_decertify_child_account"></a>

## Function `decertify_child_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_child_account">decertify_child_account</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_child_account">decertify_child_account</a>(addr: address) {
    Transaction::assert(!<a href="#0x0_VASP_is_parent_vasp">is_parent_vasp</a>(addr), 7003);
    <b>let</b> root_account_addr = <a href="account_type.md#0x0_AccountType_transition_cap_root_addr">AccountType::transition_cap_root_addr</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(Transaction::sender());
    Transaction::assert(<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_account_addr), 7001);
    <b>let</b> child_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
    child_vasp.is_certified = <b>false</b>;
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr, child_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_recertify_child_account"></a>

## Function `recertify_child_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_child_account">recertify_child_account</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_child_account">recertify_child_account</a>(addr: address) {
    <b>let</b> root_account_addr = <a href="account_type.md#0x0_AccountType_transition_cap_root_addr">AccountType::transition_cap_root_addr</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(Transaction::sender());
    Transaction::assert(<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_account_addr), 7001);
    // Child cert must be published under the child, but not yet certified
    <b>let</b> child_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr);
    child_vasp.is_certified = <b>true</b>;
    <a href="account_type.md#0x0_AccountType_update">AccountType::update</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr, child_vasp);
}
</code></pre>



</details>

<a name="0x0_VASP_apply_for_child_vasp_credential"></a>

## Function `apply_for_child_vasp_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_child_vasp_credential">apply_for_child_vasp_credential</a>(root_vasp_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_apply_for_child_vasp_credential">apply_for_child_vasp_credential</a>(root_vasp_addr: address) {
    <b>let</b> sender = Transaction::sender();
    Transaction::assert(!<a href="#0x0_VASP_is_vasp">is_vasp</a>(sender), 7002);
    Transaction::assert(<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_vasp_addr), 7001);
    <a href="account_type.md#0x0_AccountType_assert_has_transition_cap">AccountType::assert_has_transition_cap</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(root_vasp_addr);
    <a href="account_type.md#0x0_AccountType_apply_for">AccountType::apply_for</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(<a href="#0x0_VASP_ChildVASP">ChildVASP</a>{ is_certified: <b>false</b> }, root_vasp_addr);
}
</code></pre>



</details>

<a name="0x0_VASP_is_root_vasp"></a>

## Function `is_root_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(addr: address): bool {
    <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr)) {
        <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(addr);
        !<a href="#0x0_VASP_root_credential_expired">root_credential_expired</a>(&root_vasp)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x0_VASP_is_child_vasp"></a>

## Function `is_child_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(addr: address): bool {
    <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)) {
        <a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(<a href="account_type.md#0x0_AccountType_root_address">AccountType::root_address</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)) &&
        <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr).is_certified
    } <b>else</b> <b>false</b>
}
</code></pre>



</details>

<a name="0x0_VASP_root_vasp_address"></a>

## Function `root_vasp_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr: address): address {
    <b>if</b> (<a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(addr)) {
        <a href="account_type.md#0x0_AccountType_root_address">AccountType::root_address</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
    } <b>else</b> <b>if</b> (<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(addr)) {
        addr
    } <b>else</b> {
        <b>abort</b> 2
    }
}
</code></pre>



</details>

<a name="0x0_VASP_is_root_child_vasp"></a>

## Function `is_root_child_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_root_child_vasp">is_root_child_vasp</a>(root_vasp_addr: address, child_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_root_child_vasp">is_root_child_vasp</a>(root_vasp_addr: address, child_address: address): bool {
    <b>if</b> (<a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_vasp_addr) && <a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(child_address)) {
        <a href="account_type.md#0x0_AccountType_root_address">AccountType::root_address</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(child_address) == root_vasp_addr
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x0_VASP_is_parent_vasp"></a>

## Function `is_parent_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent_vasp">is_parent_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent_vasp">is_parent_vasp</a>(addr: address): bool {
    <a href="account_type.md#0x0_AccountType_has_transition_cap">AccountType::has_transition_cap</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_VASP_is_vasp"></a>

## Function `is_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_vasp">is_vasp</a>(addr: address): bool {
    <a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(addr) || <a href="#0x0_VASP_is_child_vasp">is_child_vasp</a>(addr)
}
</code></pre>



</details>

<a name="0x0_VASP_allows_child_accounts"></a>

## Function `allows_child_accounts`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_allows_child_accounts">allows_child_accounts</a>(root_vasp_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_allows_child_accounts">allows_child_accounts</a>(root_vasp_addr: address): bool {
    <a href="#0x0_VASP_is_root_vasp">is_root_vasp</a>(root_vasp_addr) &&
    <a href="account_type.md#0x0_AccountType_has_transition_cap">AccountType::has_transition_cap</a>&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(root_vasp_addr)
}
</code></pre>



</details>

<a name="0x0_VASP_human_name"></a>

## Function `human_name`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt; {
    <b>let</b> root_vasp_addr = <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr);
    *&root_vasp.human_name
}
</code></pre>



</details>

<a name="0x0_VASP_base_url"></a>

## Function `base_url`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt; {
    <b>let</b> root_vasp_addr = <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr);
    *&root_vasp.base_url
}
</code></pre>



</details>

<a name="0x0_VASP_compliance_public_key"></a>

## Function `compliance_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt; {
    <b>let</b> root_vasp_addr = <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr);
    *&root_vasp.compliance_public_key
}
</code></pre>



</details>

<a name="0x0_VASP_expiration_date"></a>

## Function `expiration_date`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(addr: address): u64 {
    <b>let</b> root_vasp_addr = <a href="#0x0_VASP_root_vasp_address">root_vasp_address</a>(addr);
    <b>let</b> root_vasp = <a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="#0x0_VASP_RootVASP">RootVASP</a>&gt;(root_vasp_addr);
    root_vasp.expiration_date
}
</code></pre>



</details>

<a name="0x0_VASP_root_credential_expired"></a>

## Function `root_credential_expired`



<pre><code><b>fun</b> <a href="#0x0_VASP_root_credential_expired">root_credential_expired</a>(root_credential: &<a href="#0x0_VASP_RootVASP">VASP::RootVASP</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_VASP_root_credential_expired">root_credential_expired</a>(root_credential: &<a href="#0x0_VASP_RootVASP">RootVASP</a>): bool {
    root_credential.<a href="#0x0_VASP_expiration_date">expiration_date</a> &lt; <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>()
}
</code></pre>



</details>

<a name="0x0_VASP_assert_sender_is_assoc_vasp_privileged"></a>

## Function `assert_sender_is_assoc_vasp_privileged`



<pre><code><b>fun</b> <a href="#0x0_VASP_assert_sender_is_assoc_vasp_privileged">assert_sender_is_assoc_vasp_privileged</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_VASP_assert_sender_is_assoc_vasp_privileged">assert_sender_is_assoc_vasp_privileged</a>() {
    Transaction::assert(<a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_VASP_CreationPrivilege">CreationPrivilege</a>&gt;(Transaction::sender()), 7000);
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
