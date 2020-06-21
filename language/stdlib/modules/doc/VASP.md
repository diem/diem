
<a name="0x1_VASP"></a>

# Module `0x1::VASP`

### Table of Contents

-  [Struct `ParentVASP`](#0x1_VASP_ParentVASP)
-  [Struct `ChildVASP`](#0x1_VASP_ChildVASP)
-  [Function `recertify_vasp`](#0x1_VASP_recertify_vasp)
-  [Function `decertify_vasp`](#0x1_VASP_decertify_vasp)
-  [Function `cert_lifetime`](#0x1_VASP_cert_lifetime)
-  [Function `publish_parent_vasp_credential`](#0x1_VASP_publish_parent_vasp_credential)
-  [Function `publish_child_vasp_credential`](#0x1_VASP_publish_child_vasp_credential)
-  [Function `parent_address`](#0x1_VASP_parent_address)
-  [Function `is_parent`](#0x1_VASP_is_parent)
-  [Function `is_child`](#0x1_VASP_is_child)
-  [Function `is_vasp`](#0x1_VASP_is_vasp)
-  [Function `human_name`](#0x1_VASP_human_name)
-  [Function `base_url`](#0x1_VASP_base_url)
-  [Function `compliance_public_key`](#0x1_VASP_compliance_public_key)
-  [Function `expiration_date`](#0x1_VASP_expiration_date)
-  [Function `num_children`](#0x1_VASP_num_children)
-  [Function `rotate_base_url`](#0x1_VASP_rotate_base_url)
-  [Function `rotate_compliance_public_key`](#0x1_VASP_rotate_compliance_public_key)
-  [Specification](#0x1_VASP_Specification)
    -  [Module specifications](#0x1_VASP_@Module_specifications)
        -  [Number of children is consistent](#0x1_VASP_@Number_of_children_is_consistent)
        -  [Number of children does not change](#0x1_VASP_@Number_of_children_does_not_change)
        -  [Specifications for individual functions](#0x1_VASP_@Specifications_for_individual_functions)
    -  [Function `recertify_vasp`](#0x1_VASP_Specification_recertify_vasp)
    -  [Function `decertify_vasp`](#0x1_VASP_Specification_decertify_vasp)
    -  [Function `publish_parent_vasp_credential`](#0x1_VASP_Specification_publish_parent_vasp_credential)
    -  [Function `publish_child_vasp_credential`](#0x1_VASP_Specification_publish_child_vasp_credential)
    -  [Function `rotate_base_url`](#0x1_VASP_Specification_rotate_base_url)
    -  [Function `rotate_compliance_public_key`](#0x1_VASP_Specification_rotate_compliance_public_key)



<a name="0x1_VASP_ParentVASP"></a>

## Struct `ParentVASP`

Each VASP has a unique root account that holds a
<code><a href="#0x1_VASP_ParentVASP">ParentVASP</a></code> resource. This resource holds
the VASP's globally unique name and all of the metadata that other VASPs need to perform
off-chain protocols with this one.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>human_name: vector&lt;u8&gt;</code>
</dt>
<dd>
 The human readable name of this VASP. Immutable.
</dd>
<dt>

<code>base_url: vector&lt;u8&gt;</code>
</dt>
<dd>
 The base_url holds the URL to be used for off-chain communication. This contains the
 entire URL (e.g. https://...). Mutable.
</dd>
<dt>

<code>expiration_date: u64</code>
</dt>
<dd>
 Expiration date in microseconds from unix epoch. For V1 VASPs, it is always set to
 U64_MAX. Mutable, but only by the Association.
</dd>
<dt>

<code>compliance_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>
 32 byte Ed25519 public key whose counterpart must be used to sign
 (1) the payment metadata for on-chain travel rule transactions
 (2) the KYC information exchanged in the off-chain travel rule protocol.
 Note that this is different than
<code>authentication_key</code> used in LibraAccount::T, which is
 a hash of a public key + signature scheme identifier, not a public key. Mutable.
</dd>
<dt>

<code>num_children: u64</code>
</dt>
<dd>
 Number of child accounts this parent has created.
</dd>
</dl>


</details>

<a name="0x1_VASP_ChildVASP"></a>

## Struct `ChildVASP`

A resource that represents a child account of the parent VASP account at
<code>parent_vasp_addr</code>


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>
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

<a name="0x1_VASP_recertify_vasp"></a>

## Function `recertify_vasp`

Renew's
<code>parent_vasp</code>'s certification


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a>) {
    parent_vasp.expiration_date = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x1_VASP_cert_lifetime">cert_lifetime</a>();
}
</code></pre>



</details>

<a name="0x1_VASP_decertify_vasp"></a>

## Function `decertify_vasp`

Non-destructively decertify
<code>parent_vasp</code>. Can be
recertified later on via
<code>recertify_vasp</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a>) {
    // Expire the parent credential.
    parent_vasp.expiration_date = 0;
}
</code></pre>



</details>

<a name="0x1_VASP_cert_lifetime"></a>

## Function `cert_lifetime`



<pre><code><b>fun</b> <a href="#0x1_VASP_cert_lifetime">cert_lifetime</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_VASP_cert_lifetime">cert_lifetime</a>(): u64 {
    31540000000000
}
</code></pre>



</details>

<a name="0x1_VASP_publish_parent_vasp_credential"></a>

## Function `publish_parent_vasp_credential`

Create a new
<code><a href="#0x1_VASP_ParentVASP">ParentVASP</a></code> resource under
<code>vasp</code>
Aborts if
<code>association</code> is not an Association account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_AssociationRootRole">Roles::AssociationRootRole</a>&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(
    vasp: &signer,
    _: &Capability&lt;AssociationRootRole&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;
) {
    <b>let</b> vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vasp);
    // TODO: proper error code
    <b>assert</b>(!exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(vasp_addr), 7000);
    <b>assert</b>(<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> compliance_public_key), 7004);
    move_to(
        vasp,
        <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
            // For testnet and V1, so it should never expire. So set <b>to</b> u64::MAX
            expiration_date: 18446744073709551615,
            human_name,
            base_url,
            compliance_public_key,
            num_children: 0
        }
    );
}
</code></pre>



</details>

<a name="0x1_VASP_publish_child_vasp_credential"></a>

## Function `publish_child_vasp_credential`

Create a child VASP resource for the
<code>parent</code>
Aborts if
<code>parent</code> is not a ParentVASP


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_ParentVASPRole">Roles::ParentVASPRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(
    parent: &signer,
    child: &signer,
    _: &Capability&lt;ParentVASPRole&gt;,
) <b>acquires</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    <b>let</b> child_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(child);
    // TODO: proper error code
    <b>assert</b>(!exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(child_vasp_addr), 7000);
    <b>let</b> parent_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(parent);
    <b>assert</b>(exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_vasp_addr), 7000);
    <b>let</b> num_children = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_vasp_addr).num_children;
    *num_children = *num_children + 1;
    move_to(child, <a href="#0x1_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr });
}
</code></pre>



</details>

<a name="0x1_VASP_parent_address"></a>

## Function `parent_address`

Return
<code>addr</code> if
<code>addr</code> is a
<code><a href="#0x1_VASP_ParentVASP">ParentVASP</a></code> or its parent's address if it is a
<code><a href="#0x1_VASP_ChildVASP">ChildVASP</a></code>
Aborts otherwise


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_parent_address">parent_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_parent_address">parent_address</a>(addr: address): address <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a> {
    <b>if</b> (exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)) {
        addr
    } <b>else</b> <b>if</b> (exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr)) {
        borrow_global&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    } <b>else</b> { // wrong account type, <b>abort</b>
        <b>abort</b>(88)
    }
}
</code></pre>



</details>

<a name="0x1_VASP_is_parent"></a>

## Function `is_parent`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_parent">is_parent</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_parent">is_parent</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_VASP_is_child"></a>

## Function `is_child`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_child">is_child</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_child">is_child</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_VASP_is_vasp"></a>

## Function `is_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool {
    <a href="#0x1_VASP_is_parent">is_parent</a>(addr) || <a href="#0x1_VASP_is_child">is_child</a>(addr)
}
</code></pre>



</details>

<a name="0x1_VASP_human_name"></a>

## Function `human_name`

Return the human-readable name for the VASP account
Aborts if
<code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt;  <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)).human_name
}
</code></pre>



</details>

<a name="0x1_VASP_base_url"></a>

## Function `base_url`

Return the base URL for the VASP account
Aborts if
<code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt;  <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)).base_url
}
</code></pre>



</details>

<a name="0x1_VASP_compliance_public_key"></a>

## Function `compliance_public_key`

Return the compliance public key for the VASP account
Aborts if
<code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)).compliance_public_key
}
</code></pre>



</details>

<a name="0x1_VASP_expiration_date"></a>

## Function `expiration_date`

Return the expiration date for the VASP account
Aborts if
<code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_expiration_date">expiration_date</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_expiration_date">expiration_date</a>(addr: address): u64  <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)).expiration_date
}
</code></pre>



</details>

<a name="0x1_VASP_num_children"></a>

## Function `num_children`

Return the number of child accounts for this VASP.
The total number of accounts for this VASP is num_children() + 1
Aborts if
<code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_num_children">num_children</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_num_children">num_children</a>(addr: address): u64  <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)).num_children
}
</code></pre>



</details>

<a name="0x1_VASP_rotate_base_url"></a>

## Function `rotate_base_url`

Rotate the base URL for the
<code>parent_vasp</code> account to
<code>new_url</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_base_url">rotate_base_url</a>(parent_vasp: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_base_url">rotate_base_url</a>(parent_vasp: &signer, new_url: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    <b>let</b> parent_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(parent_vasp);
    borrow_global_mut&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_addr).base_url = new_url
}
</code></pre>



</details>

<a name="0x1_VASP_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`

Rotate the compliance public key for
<code>parent_vasp</code> to
<code>new_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(parent_vasp: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(
    parent_vasp: &signer,
    new_key: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    <b>assert</b>(<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> new_key), 7004);
    <b>let</b> parent_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(parent_vasp);
    borrow_global_mut&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_addr).compliance_public_key = new_key
}
</code></pre>



</details>

<a name="0x1_VASP_Specification"></a>

## Specification


<a name="0x1_VASP_@Module_specifications"></a>

### Module specifications


> TODO(emmazzz): verification is turned off because
<code><a href="Signature.md#0x1_Signature">Signature</a></code> module
> has not been specified. See issue #4666.


<pre><code>pragma verify = <b>false</b>;
</code></pre>



Returns true if
<code>addr</code> is a VASP.


<a name="0x1_VASP_spec_is_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr: address): bool {
    <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr) || <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr)
}
</code></pre>


Returns true if
<code>addr</code> is a ParentVASP.


<a name="0x1_VASP_spec_is_parent_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)
}
</code></pre>


Returns true if
<code>addr</code> is a ChildVASP.


<a name="0x1_VASP_spec_is_child_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
}
</code></pre>


Returns the number of children under
<code>parent</code>.


<a name="0x1_VASP_spec_get_num_children"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent: address): u64 {
    <b>global</b>&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent).num_children
}
</code></pre>


Returns the parent address of a VASP.


<a name="0x1_VASP_spec_parent_address"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr: address): address {
    <b>if</b> (exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)) {
        addr
    } <b>else</b> {
        <b>global</b>&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    }
}
<a name="0x1_VASP_spec_cert_lifetime"></a>
<b>define</b> <a href="#0x1_VASP_spec_cert_lifetime">spec_cert_lifetime</a>(): u64 {
    31540000000000
}
<a name="0x1_VASP_spec_root_address"></a>
<b>define</b> <a href="#0x1_VASP_spec_root_address">spec_root_address</a>(): address {
    0xA550C18
}
</code></pre>



<a name="0x1_VASP_@Number_of_children_is_consistent"></a>

#### Number of children is consistent

> PROVER TODO(emmazzz): implement the features that allows users
> to reason about number of resources with certain property,
> such as "number of ChildVASPs whose parent address is 0xDD".
> See issue #4665.

<a name="0x1_VASP_@Number_of_children_does_not_change"></a>

#### Number of children does not change



<a name="0x1_VASP_NumChildrenRemainsSame"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> {
    <b>ensures</b> forall parent: address
        where <b>old</b>(<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(parent)):
            <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent))
             == <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> <b>to</b> * <b>except</b> publish_child_vasp_credential;
</code></pre>



<a name="0x1_VASP_@Specifications_for_individual_functions"></a>

#### Specifications for individual functions



<a name="0x1_VASP_AbortsIfNotVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> {
    addr: address;
    <b>aborts_if</b> !<a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> <b>to</b> parent_address, human_name, base_url,
    compliance_public_key, expiration_date, num_children;
</code></pre>




<a name="0x1_VASP_AbortsIfParentIsNotParentVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> {
    addr: address;
    <b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr));
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> <b>to</b> human_name, base_url,
    compliance_public_key, expiration_date, num_children;
</code></pre>



<a name="0x1_VASP_Specification_recertify_vasp"></a>

### Function `recertify_vasp`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="LibraTimestamp.md#0x1_LibraTimestamp_CurrentTimeMicroseconds">LibraTimestamp::CurrentTimeMicroseconds</a>&gt;(<a href="#0x1_VASP_spec_root_address">spec_root_address</a>());
<b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_assoc_unix_time">LibraTimestamp::assoc_unix_time</a>() + <a href="#0x1_VASP_spec_cert_lifetime">spec_cert_lifetime</a>() &gt; max_u64();
<b>ensures</b> parent_vasp.expiration_date
     == <a href="LibraTimestamp.md#0x1_LibraTimestamp_assoc_unix_time">LibraTimestamp::assoc_unix_time</a>() + <a href="#0x1_VASP_spec_cert_lifetime">spec_cert_lifetime</a>();
</code></pre>



<a name="0x1_VASP_Specification_decertify_vasp"></a>

### Function `decertify_vasp`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x1_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> parent_vasp.expiration_date == 0;
</code></pre>



<a name="0x1_VASP_Specification_publish_parent_vasp_credential"></a>

### Function `publish_parent_vasp_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_AssociationRootRole">Roles::AssociationRootRole</a>&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(vasp)) == 0;
</code></pre>



<a name="0x1_VASP_Specification_publish_child_vasp_credential"></a>

### Function `publish_child_vasp_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_ParentVASPRole">Roles::ParentVASPRole</a>&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(child));
<b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent));
<b>aborts_if</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent)) + 1
        &gt; max_u64();
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent))
     == <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent))) + 1;
<b>ensures</b> <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(child));
<b>ensures</b> <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(child))
     == <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent);
</code></pre>



<a name="0x1_VASP_Specification_rotate_base_url"></a>

### Function `rotate_base_url`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_base_url">rotate_base_url</a>(parent_vasp: &signer, new_url: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent_vasp));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent_vasp)).base_url
     == new_url;
</code></pre>



<a name="0x1_VASP_Specification_rotate_compliance_public_key"></a>

### Function `rotate_compliance_public_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(parent_vasp: &signer, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent_vasp));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(parent_vasp)).compliance_public_key
     == new_key;
</code></pre>
