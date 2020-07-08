
<a name="0x1_DualAttestation"></a>

# Module `0x1::DualAttestation`

### Table of Contents

-  [Resource `Credential`](#0x1_DualAttestation_Credential)
-  [Function `publish_credential`](#0x1_DualAttestation_publish_credential)
-  [Function `rotate_base_url`](#0x1_DualAttestation_rotate_base_url)
-  [Function `rotate_compliance_public_key`](#0x1_DualAttestation_rotate_compliance_public_key)
-  [Function `human_name`](#0x1_DualAttestation_human_name)
-  [Function `base_url`](#0x1_DualAttestation_base_url)
-  [Function `compliance_public_key`](#0x1_DualAttestation_compliance_public_key)
-  [Function `expiration_date`](#0x1_DualAttestation_expiration_date)
-  [Function `recertify`](#0x1_DualAttestation_recertify)
-  [Function `decertify`](#0x1_DualAttestation_decertify)
-  [Function `cert_lifetime`](#0x1_DualAttestation_cert_lifetime)
-  [Specification](#0x1_DualAttestation_Specification)
    -  [Function `rotate_base_url`](#0x1_DualAttestation_Specification_rotate_base_url)
    -  [Function `rotate_compliance_public_key`](#0x1_DualAttestation_Specification_rotate_compliance_public_key)
    -  [Function `recertify`](#0x1_DualAttestation_Specification_recertify)
    -  [Function `decertify`](#0x1_DualAttestation_Specification_decertify)



<a name="0x1_DualAttestation_Credential"></a>

## Resource `Credential`

This resource holds an entity's globally unique name and all of the metadata it needs to
participate if off-chain protocols.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DualAttestation_Credential">Credential</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>human_name: vector&lt;u8&gt;</code>
</dt>
<dd>
 The human readable name of this entity. Immutable.
</dd>
<dt>

<code>base_url: vector&lt;u8&gt;</code>
</dt>
<dd>
 The base_url holds the URL to be used for off-chain communication. This contains the
 entire URL (e.g. https://...). Mutable.
</dd>
<dt>

<code>compliance_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>
 32 byte Ed25519 public key whose counterpart must be used to sign
 (1) the payment metadata for on-chain transactions that require dual attestation (e.g.,
     transactions subject to the travel rule)
 (2) information exchanged in the off-chain protocols (e.g., KYC info in the travel rule
     protocol)
 Note that this is different than
<code>authentication_key</code> used in LibraAccount::T, which is
 a hash of a public key + signature scheme identifier, not a public key. Mutable.
</dd>
<dt>

<code>expiration_date: u64</code>
</dt>
<dd>
 Expiration date in microseconds from unix epoch. For V1, it is always set to
 U64_MAX. Mutable, but only by LibraRoot.
</dd>
</dl>


</details>

<a name="0x1_DualAttestation_publish_credential"></a>

## Function `publish_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_publish_credential">publish_credential</a>(created: &signer, creator: &signer, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_publish_credential">publish_credential</a>(
    created: &signer,
    creator: &signer,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
) {
    <b>assert</b>(
        <a href="Roles.md#0x1_Roles_has_parent_VASP_role">Roles::has_parent_VASP_role</a>(created) || <a href="Roles.md#0x1_Roles_has_designated_dealer_role">Roles::has_designated_dealer_role</a>(created),
        ENOT_PARENT_VASP_OR_DD
    );
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(creator), ENOT_LIBRA_ROOT);
    <b>assert</b>(<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> compliance_public_key), EINVALID_PUBLIC_KEY);
    move_to(created, <a href="#0x1_DualAttestation_Credential">Credential</a> {
        human_name,
        base_url,
        compliance_public_key,
        // For testnet and V1, so it should never expire. So set <b>to</b> u64::MAX
        expiration_date: U64_MAX,
    })
}
</code></pre>



</details>

<a name="0x1_DualAttestation_rotate_base_url"></a>

## Function `rotate_base_url`

Rotate the base URL for
<code>account</code> to
<code>new_url</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_base_url">rotate_base_url</a>(account: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_base_url">rotate_base_url</a>(account: &signer, new_url: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    borrow_global_mut&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).base_url = new_url
}
</code></pre>



</details>

<a name="0x1_DualAttestation_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`

Rotate the compliance public key for
<code>account</code> to
<code>new_key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_compliance_public_key">rotate_compliance_public_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_compliance_public_key">rotate_compliance_public_key</a>(
    account: &signer,
    new_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    <b>assert</b>(<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> new_key), EINVALID_PUBLIC_KEY);
    <b>let</b> parent_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    borrow_global_mut&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(parent_addr).compliance_public_key = new_key
}
</code></pre>



</details>

<a name="0x1_DualAttestation_human_name"></a>

## Function `human_name`

Return the human-readable name for the VASP account.
Aborts if
<code>addr</code> does not have a
<code><a href="#0x1_DualAttestation_Credential">Credential</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_human_name">human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_human_name">human_name</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    *&borrow_global&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(addr).human_name
}
</code></pre>



</details>

<a name="0x1_DualAttestation_base_url"></a>

## Function `base_url`

Return the base URL for
<code>addr</code>.
Aborts if
<code>addr</code> does not have a
<code><a href="#0x1_DualAttestation_Credential">Credential</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_base_url">base_url</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_base_url">base_url</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    *&borrow_global&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(addr).base_url
}
</code></pre>



</details>

<a name="0x1_DualAttestation_compliance_public_key"></a>

## Function `compliance_public_key`

Return the compliance public key for
<code>addr</code>.
Aborts if
<code>addr</code> does not have a
<code><a href="#0x1_DualAttestation_Credential">Credential</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    *&borrow_global&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(addr).compliance_public_key
}
</code></pre>



</details>

<a name="0x1_DualAttestation_expiration_date"></a>

## Function `expiration_date`

Return the expiration date
<code>addr
Aborts <b>if</b> </code>addr
<code> does not have a </code>Credential` resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_expiration_date">expiration_date</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_expiration_date">expiration_date</a>(addr: address): u64  <b>acquires</b> <a href="#0x1_DualAttestation_Credential">Credential</a> {
    *&borrow_global&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(addr).expiration_date
}
</code></pre>



</details>

<a name="0x1_DualAttestation_recertify"></a>

## Function `recertify`

Renew's
<code>credential</code>'s certificate


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_recertify">recertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">DualAttestation::Credential</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_recertify">recertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">Credential</a>) {
    credential.expiration_date = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() + <a href="#0x1_DualAttestation_cert_lifetime">cert_lifetime</a>();
}
</code></pre>



</details>

<a name="0x1_DualAttestation_decertify"></a>

## Function `decertify`

Non-destructively decertify
<code>credential</code>. Can be recertified later on via
<code>recertify</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_decertify">decertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">DualAttestation::Credential</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_decertify">decertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">Credential</a>) {
    // Expire the parent credential.
    credential.expiration_date = 0;
}
</code></pre>



</details>

<a name="0x1_DualAttestation_cert_lifetime"></a>

## Function `cert_lifetime`

A year in microseconds


<pre><code><b>fun</b> <a href="#0x1_DualAttestation_cert_lifetime">cert_lifetime</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DualAttestation_cert_lifetime">cert_lifetime</a>(): u64 {
    31540000000000
}
</code></pre>



</details>

<a name="0x1_DualAttestation_Specification"></a>

## Specification


<a name="0x1_DualAttestation_Specification_rotate_base_url"></a>

### Function `rotate_base_url`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_base_url">rotate_base_url</a>(account: &signer, new_url: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b>
    <b>global</b>&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).base_url == new_url;
</code></pre>



<a name="0x1_DualAttestation_Specification_rotate_compliance_public_key"></a>

### Function `rotate_compliance_public_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_rotate_compliance_public_key">rotate_compliance_public_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_spec_ed25519_validate_pubkey">Signature::spec_ed25519_validate_pubkey</a>(new_key);
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).compliance_public_key
     == new_key;
</code></pre>



Spec version of
<code><a href="#0x1_DualAttestation_compliance_public_key">Self::compliance_public_key</a></code>.


<a name="0x1_DualAttestation_spec_compliance_public_key"></a>


<pre><code><b>define</b> <a href="#0x1_DualAttestation_spec_compliance_public_key">spec_compliance_public_key</a>(addr: address): vector&lt;u8&gt; {
    <b>global</b>&lt;<a href="#0x1_DualAttestation_Credential">Credential</a>&gt;(addr).compliance_public_key
}
</code></pre>



<a name="0x1_DualAttestation_Specification_recertify"></a>

### Function `recertify`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_recertify">recertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">DualAttestation::Credential</a>)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_root_ctm_initialized">LibraTimestamp::root_ctm_initialized</a>();
<b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() + <a href="#0x1_DualAttestation_spec_cert_lifetime">spec_cert_lifetime</a>() &gt; max_u64();
<b>ensures</b> credential.expiration_date
     == <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() + <a href="#0x1_DualAttestation_spec_cert_lifetime">spec_cert_lifetime</a>();
</code></pre>



<a name="0x1_DualAttestation_Specification_decertify"></a>

### Function `decertify`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestation_decertify">decertify</a>(credential: &<b>mut</b> <a href="#0x1_DualAttestation_Credential">DualAttestation::Credential</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> credential.expiration_date == 0;
</code></pre>




<a name="0x1_DualAttestation_spec_cert_lifetime"></a>


<pre><code><b>define</b> <a href="#0x1_DualAttestation_spec_cert_lifetime">spec_cert_lifetime</a>(): u64 {
    31540000000000
}
</code></pre>
