
<a name="0x0_VASP"></a>

# Module `0x0::VASP`

### Table of Contents

-  [Struct `ParentVASP`](#0x0_VASP_ParentVASP)
-  [Struct `ChildVASP`](#0x0_VASP_ChildVASP)
-  [Function `recertify_vasp`](#0x0_VASP_recertify_vasp)
-  [Function `decertify_vasp`](#0x0_VASP_decertify_vasp)
-  [Function `publish_parent_vasp_credential`](#0x0_VASP_publish_parent_vasp_credential)
-  [Function `publish_child_vasp_credential`](#0x0_VASP_publish_child_vasp_credential)
-  [Function `parent_address`](#0x0_VASP_parent_address)
-  [Function `is_parent`](#0x0_VASP_is_parent)
-  [Function `is_child`](#0x0_VASP_is_child)
-  [Function `is_vasp`](#0x0_VASP_is_vasp)
-  [Function `human_name`](#0x0_VASP_human_name)
-  [Function `base_url`](#0x0_VASP_base_url)
-  [Function `compliance_public_key`](#0x0_VASP_compliance_public_key)
-  [Function `expiration_date`](#0x0_VASP_expiration_date)
-  [Function `rotate_base_url`](#0x0_VASP_rotate_base_url)
-  [Function `rotate_compliance_public_key`](#0x0_VASP_rotate_compliance_public_key)
-  [Function `cert_lifetime`](#0x0_VASP_cert_lifetime)



<a name="0x0_VASP_ParentVASP"></a>

## Struct `ParentVASP`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>
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

<a name="0x0_VASP_recertify_vasp"></a>

## Function `recertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_recertify_vasp">recertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>) {
    parent_vasp.expiration_date = <a href="LibraTimestamp.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x0_VASP_cert_lifetime">cert_lifetime</a>();
}
</code></pre>



</details>

<a name="0x0_VASP_decertify_vasp"></a>

## Function `decertify_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">VASP::ParentVASP</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_decertify_vasp">decertify_vasp</a>(parent_vasp: &<b>mut</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a>) {
    // Expire the parent credential.
    parent_vasp.expiration_date = 0;
}
</code></pre>



</details>

<a name="0x0_VASP_publish_parent_vasp_credential"></a>

## Function `publish_parent_vasp_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(association: &signer, vasp: &signer, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(
    association: &signer,
    vasp: &signer,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;
) {
    <a href="Association.md#0x0_Association_assert_is_root">Association::assert_is_root</a>(association);
    <b>assert</b>(<a href="Signature.md#0x0_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> compliance_public_key), 7004);
    move_to(
        vasp,
        <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
            // For testnet, so it should never expire. So set <b>to</b> u64::MAX
            expiration_date: 18446744073709551615,
            human_name,
            base_url,
            compliance_public_key,
        }
    )
}
</code></pre>



</details>

<a name="0x0_VASP_publish_child_vasp_credential"></a>

## Function `publish_child_vasp_credential`

Create a child VASP resource for the
<code>parent</code>
Aborts if
<code>parent</code> is not a ParentVASP


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer) {
    <b>let</b> parent_vasp_addr = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(parent);
    <b>assert</b>(exists&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(parent_vasp_addr), 7000);
    move_to(child, <a href="#0x0_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr });
}
</code></pre>



</details>

<a name="0x0_VASP_parent_address"></a>

## Function `parent_address`

Return
<code>addr</code> if
<code>addr</code> is a
<code><a href="#0x0_VASP_ParentVASP">ParentVASP</a></code> or its parent's address if it is a
<code><a href="#0x0_VASP_ChildVASP">ChildVASP</a></code>
Aborts otherwise


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_parent_address">parent_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_parent_address">parent_address</a>(addr: address): address <b>acquires</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a> {
    <b>if</b> (exists&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(addr)) {
        addr
    } <b>else</b> <b>if</b> (exists&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)) {
        borrow_global&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    } <b>else</b> { // wrong account type, <b>abort</b>
        <b>abort</b>(88)
    }
}
</code></pre>



</details>

<a name="0x0_VASP_is_parent"></a>

## Function `is_parent`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent">is_parent</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_parent">is_parent</a>(addr: address): bool {
    exists&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_VASP_is_child"></a>

## Function `is_child`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_child">is_child</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_is_child">is_child</a>(addr: address): bool {
    exists&lt;<a href="#0x0_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
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
    <a href="#0x0_VASP_is_parent">is_parent</a>(addr) || <a href="#0x0_VASP_is_child">is_child</a>(addr)
}
</code></pre>



</details>

<a name="0x0_VASP_human_name"></a>

## Function `human_name`

Return the human-readable name for the VASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_human_name">human_name</a>(addr: address): vector&lt;u8&gt;  <b>acquires</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>, <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x0_VASP_parent_address">parent_address</a>(addr)).human_name
}
</code></pre>



</details>

<a name="0x0_VASP_base_url"></a>

## Function `base_url`

Return the base URL for the VASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_base_url">base_url</a>(addr: address): vector&lt;u8&gt;  <b>acquires</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>, <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x0_VASP_parent_address">parent_address</a>(addr)).base_url
}
</code></pre>



</details>

<a name="0x0_VASP_compliance_public_key"></a>

## Function `compliance_public_key`

Return the compliance public key for the VASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>, <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x0_VASP_parent_address">parent_address</a>(addr)).compliance_public_key
}
</code></pre>



</details>

<a name="0x0_VASP_expiration_date"></a>

## Function `expiration_date`

Return the expiration date for the VASP account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_expiration_date">expiration_date</a>(addr: address): u64  <b>acquires</b> <a href="#0x0_VASP_ChildVASP">ChildVASP</a>, <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    *&borrow_global&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="#0x0_VASP_parent_address">parent_address</a>(addr)).expiration_date
}
</code></pre>



</details>

<a name="0x0_VASP_rotate_base_url"></a>

## Function `rotate_base_url`

Rotate the base URL for the
<code>parent_vasp</code> account to
<code>new_url</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_base_url">rotate_base_url</a>(parent_vasp: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_base_url">rotate_base_url</a>(parent_vasp: &signer, new_url: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    <b>let</b> parent_addr = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(parent_vasp);
    borrow_global_mut&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(parent_addr).base_url = new_url
}
</code></pre>



</details>

<a name="0x0_VASP_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`

Rotate the compliance public key for
<code>parent_vasp</code> to
<code>new_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(parent_vasp: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASP_rotate_compliance_public_key">rotate_compliance_public_key</a>(
    parent_vasp: &signer,
    new_key: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_VASP_ParentVASP">ParentVASP</a> {
    <b>assert</b>(<a href="Signature.md#0x0_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> new_key), 7004);
    <b>let</b> parent_addr = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(parent_vasp);
    borrow_global_mut&lt;<a href="#0x0_VASP_ParentVASP">ParentVASP</a>&gt;(parent_addr).compliance_public_key = new_key
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
