
<a name="0x1_DiemId"></a>

# Module `0x1::DiemId`

Module managing Diem ID.


-  [Resource `DiemIdDomains`](#0x1_DiemId_DiemIdDomains)
-  [Struct `DiemIdDomain`](#0x1_DiemId_DiemIdDomain)
-  [Resource `DiemIdDomainManager`](#0x1_DiemId_DiemIdDomainManager)
-  [Struct `DiemIdDomainEvent`](#0x1_DiemId_DiemIdDomainEvent)
-  [Constants](#@Constants_0)
-  [Function `create_diem_id_domain`](#0x1_DiemId_create_diem_id_domain)
-  [Function `publish_diem_id_domains`](#0x1_DiemId_publish_diem_id_domains)
-  [Function `has_diem_id_domains`](#0x1_DiemId_has_diem_id_domains)
-  [Function `publish_diem_id_domain_manager`](#0x1_DiemId_publish_diem_id_domain_manager)
-  [Function `add_diem_id_domain`](#0x1_DiemId_add_diem_id_domain)
-  [Function `remove_diem_id_domain`](#0x1_DiemId_remove_diem_id_domain)
-  [Function `has_diem_id_domain`](#0x1_DiemId_has_diem_id_domain)
-  [Function `tc_domain_manager_exists`](#0x1_DiemId_tc_domain_manager_exists)


<pre><code><b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemId_DiemIdDomains"></a>

## Resource `DiemIdDomains`

This resource holds an entity's domain names needed to send and receive payments using diem IDs.


<pre><code><b>struct</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>domains: vector&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemId::DiemIdDomain</a>&gt;</code>
</dt>
<dd>
 The list of domain names owned by this parent vasp account
</dd>
</dl>


</details>

<details>
<summary>Specification</summary>


All <code><a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a></code>s stored in the <code><a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a></code> resource are no more than 63 characters long.


<pre><code><b>invariant</b> <b>forall</b> i in 0..len(domains): len(domains[i].domain) &lt;= <a href="DiemId.md#0x1_DiemId_DOMAIN_LENGTH">DOMAIN_LENGTH</a>;
</code></pre>


The list of <code><a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a></code>s are a set


<pre><code><b>invariant</b> <b>forall</b> i in 0..len(domains):
    <b>forall</b> j in i + 1..len(domains): domains[i] != domains[j];
</code></pre>



</details>

<a name="0x1_DiemId_DiemIdDomain"></a>

## Struct `DiemIdDomain`

Struct to store the limit on-chain


<pre><code><b>struct</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>domain: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<details>
<summary>Specification</summary>


All <code><a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a></code>s must be no more than 63 characters long.


<pre><code><b>invariant</b> len(domain) &lt;= <a href="DiemId.md#0x1_DiemId_DOMAIN_LENGTH">DOMAIN_LENGTH</a>;
</code></pre>



</details>

<a name="0x1_DiemId_DiemIdDomainManager"></a>

## Resource `DiemIdDomainManager`



<pre><code><b>struct</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>diem_id_domain_events: <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemId::DiemIdDomainEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for <code>domains</code> added or removed events. Emitted every time a domain is added
 or removed to <code>domains</code>
</dd>
</dl>


</details>

<a name="0x1_DiemId_DiemIdDomainEvent"></a>

## Struct `DiemIdDomainEvent`



<pre><code><b>struct</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>removed: bool</code>
</dt>
<dd>
 Whether a domain was added or removed
</dd>
<dt>
<code>domain: <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemId::DiemIdDomain</a></code>
</dt>
<dd>
 Diem ID Domain string of the account
</dd>
<dt>
<code><b>address</b>: <b>address</b></code>
</dt>
<dd>
 On-chain account address
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemId_DOMAIN_LENGTH"></a>



<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_DOMAIN_LENGTH">DOMAIN_LENGTH</a>: u64 = 63;
</code></pre>



<a name="0x1_DiemId_EDIEM_ID_DOMAIN"></a>

DiemIdDomains resource is not or already published.


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN">EDIEM_ID_DOMAIN</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemId_EDIEM_ID_DOMAINS_NOT_PUBLISHED"></a>

DiemIdDomains resource was not published for a VASP account


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAINS_NOT_PUBLISHED">EDIEM_ID_DOMAINS_NOT_PUBLISHED</a>: u64 = 4;
</code></pre>



<a name="0x1_DiemId_EDIEM_ID_DOMAIN_MANAGER"></a>

DiemIdDomainManager resource is not or already published.


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN_MANAGER">EDIEM_ID_DOMAIN_MANAGER</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemId_EDOMAIN_ALREADY_EXISTS"></a>

DiemID domain already exists


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EDOMAIN_ALREADY_EXISTS">EDOMAIN_ALREADY_EXISTS</a>: u64 = 3;
</code></pre>



<a name="0x1_DiemId_EDOMAIN_NOT_FOUND"></a>

DiemID domain was not found


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EDOMAIN_NOT_FOUND">EDOMAIN_NOT_FOUND</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemId_EINVALID_DIEM_ID_DOMAIN"></a>

Invalid domain for DiemIdDomain


<pre><code><b>const</b> <a href="DiemId.md#0x1_DiemId_EINVALID_DIEM_ID_DOMAIN">EINVALID_DIEM_ID_DOMAIN</a>: u64 = 5;
</code></pre>



<a name="0x1_DiemId_create_diem_id_domain"></a>

## Function `create_diem_id_domain`



<pre><code><b>fun</b> <a href="DiemId.md#0x1_DiemId_create_diem_id_domain">create_diem_id_domain</a>(domain: vector&lt;u8&gt;): <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemId::DiemIdDomain</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemId.md#0x1_DiemId_create_diem_id_domain">create_diem_id_domain</a>(domain: vector&lt;u8&gt;): <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> {
    <b>assert</b>!(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(&domain) &lt;= <a href="DiemId.md#0x1_DiemId_DOMAIN_LENGTH">DOMAIN_LENGTH</a>, <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemId.md#0x1_DiemId_EINVALID_DIEM_ID_DOMAIN">EINVALID_DIEM_ID_DOMAIN</a>));
    <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a>{ domain }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemId.md#0x1_DiemId_CreateDiemIdDomainAbortsIf">CreateDiemIdDomainAbortsIf</a>;
<b>ensures</b> result == <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain };
</code></pre>




<a name="0x1_DiemId_CreateDiemIdDomainAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_CreateDiemIdDomainAbortsIf">CreateDiemIdDomainAbortsIf</a> {
    domain: vector&lt;u8&gt;;
    <b>aborts_if</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_length">Vector::length</a>(domain) &gt; <a href="DiemId.md#0x1_DiemId_DOMAIN_LENGTH">DOMAIN_LENGTH</a> <b>with</b> Errors::INVALID_ARGUMENT;
}
</code></pre>



</details>

<a name="0x1_DiemId_publish_diem_id_domains"></a>

## Function `publish_diem_id_domains`

Publish a <code><a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a></code> resource under <code>created</code> with an empty <code>domains</code>.
Before sending or receiving any payments using Diem IDs, the Treasury Compliance account must send
a transaction that invokes <code>add_domain_id</code> to set the <code>domains</code> field with a valid domain


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_publish_diem_id_domains">publish_diem_id_domains</a>(vasp_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_publish_diem_id_domains">publish_diem_id_domains</a>(
    vasp_account: &signer,
) {
    <a href="Roles.md#0x1_Roles_assert_parent_vasp_role">Roles::assert_parent_vasp_role</a>(vasp_account);
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vasp_account)),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN">EDIEM_ID_DOMAIN</a>)
    );
    <b>move_to</b>(vasp_account, <a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a> {
        domains: <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> vasp_addr = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vasp_account);
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotParentVasp">Roles::AbortsIfNotParentVasp</a>{account: vasp_account};
<b>include</b> <a href="DiemId.md#0x1_DiemId_PublishDiemIdDomainsAbortsIf">PublishDiemIdDomainsAbortsIf</a>;
<b>include</b> <a href="DiemId.md#0x1_DiemId_PublishDiemIdDomainsEnsures">PublishDiemIdDomainsEnsures</a>;
</code></pre>




<a name="0x1_DiemId_PublishDiemIdDomainsAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_PublishDiemIdDomainsAbortsIf">PublishDiemIdDomainsAbortsIf</a> {
    vasp_addr: <b>address</b>;
    <b>aborts_if</b> <a href="DiemId.md#0x1_DiemId_has_diem_id_domains">has_diem_id_domains</a>(vasp_addr) <b>with</b> Errors::ALREADY_PUBLISHED;
}
</code></pre>




<a name="0x1_DiemId_PublishDiemIdDomainsEnsures"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_PublishDiemIdDomainsEnsures">PublishDiemIdDomainsEnsures</a> {
    vasp_addr: <b>address</b>;
    <b>ensures</b> <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(vasp_addr);
    <b>ensures</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(<b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(vasp_addr).domains);
}
</code></pre>



</details>

<a name="0x1_DiemId_has_diem_id_domains"></a>

## Function `has_diem_id_domains`



<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_has_diem_id_domains">has_diem_id_domains</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_has_diem_id_domains">has_diem_id_domains</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr);
</code></pre>



</details>

<a name="0x1_DiemId_publish_diem_id_domain_manager"></a>

## Function `publish_diem_id_domain_manager`

Publish a <code><a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a></code> resource under <code>tc_account</code> with an empty <code>diem_id_domain_events</code>.
When Treasury Compliance account sends a transaction that invokes either <code>add_diem_id_domain</code> or
<code>remove_diem_id_domain</code>, a <code><a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a></code> is emitted and added to <code>diem_id_domain_events</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_publish_diem_id_domain_manager">publish_diem_id_domain_manager</a>(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_publish_diem_id_domain_manager">publish_diem_id_domain_manager</a>(
    tc_account : &signer,
) {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account)),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN_MANAGER">EDIEM_ID_DOMAIN_MANAGER</a>)
    );
    <b>move_to</b>(
        tc_account,
        <a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a> {
            diem_id_domain_events: <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a>&gt;(tc_account),
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>aborts_if</b> <a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>() <b>with</b> Errors::ALREADY_PUBLISHED;
<b>ensures</b> <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account));
<b>modifies</b> <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account));
</code></pre>



</details>

<a name="0x1_DiemId_add_diem_id_domain"></a>

## Function `add_diem_id_domain`

Add a DiemIdDomain to a parent VASP's DiemIdDomains resource.
When updating DiemIdDomains, a simple duplicate domain check is done.
However, since domains are case insensitive, it is possible by error that two same domains in
different lowercase and uppercase format gets added.


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_add_diem_id_domain">add_diem_id_domain</a>(tc_account: &signer, <b>address</b>: <b>address</b>, domain: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_add_diem_id_domain">add_diem_id_domain</a>(
    tc_account: &signer,
    <b>address</b>: <b>address</b>,
    domain: vector&lt;u8&gt;,
) <b>acquires</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>, <a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>!(<a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>(), <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN_MANAGER">EDIEM_ID_DOMAIN_MANAGER</a>));
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAINS_NOT_PUBLISHED">EDIEM_ID_DOMAINS_NOT_PUBLISHED</a>)
    );

    <b>let</b> account_domains = <b>borrow_global_mut</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>);
    <b>let</b> diem_id_domain = <a href="DiemId.md#0x1_DiemId_create_diem_id_domain">create_diem_id_domain</a>(domain);

    <b>assert</b>!(
        !<a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(&account_domains.domains, &diem_id_domain),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemId.md#0x1_DiemId_EDOMAIN_ALREADY_EXISTS">EDOMAIN_ALREADY_EXISTS</a>)
    );

    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> account_domains.domains, <b>copy</b> diem_id_domain);

    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance).diem_id_domain_events,
        <a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a> {
            removed: <b>false</b>,
            domain: diem_id_domain,
            <b>address</b>,
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainAbortsIf">AddDiemIdDomainAbortsIf</a>;
<b>include</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainEnsures">AddDiemIdDomainEnsures</a>;
<b>include</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainEmits">AddDiemIdDomainEmits</a>;
</code></pre>




<a name="0x1_DiemId_AddDiemIdDomainAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainAbortsIf">AddDiemIdDomainAbortsIf</a> {
    tc_account: signer;
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> domains = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>).domains;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
    <b>include</b> <a href="DiemId.md#0x1_DiemId_CreateDiemIdDomainAbortsIf">CreateDiemIdDomainAbortsIf</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>) <b>with</b> Errors::NOT_PUBLISHED;
    <b>aborts_if</b> !<a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>() <b>with</b> Errors::NOT_PUBLISHED;
    <b>aborts_if</b> contains(domains, <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain }) <b>with</b> Errors::INVALID_ARGUMENT;
}
</code></pre>




<a name="0x1_DiemId_AddDiemIdDomainEnsures"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainEnsures">AddDiemIdDomainEnsures</a> {
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> <b>post</b> domains = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>).domains;
    <b>ensures</b> contains(domains, <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain });
}
</code></pre>




<a name="0x1_DiemId_AddDiemIdDomainEmits"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_AddDiemIdDomainEmits">AddDiemIdDomainEmits</a> {
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> handle = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance).diem_id_domain_events;
    <b>let</b> msg = <a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a> {
        removed: <b>false</b>,
        domain: <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain },
        <b>address</b>,
    };
    <b>emits</b> msg <b>to</b> handle;
}
</code></pre>



</details>

<a name="0x1_DiemId_remove_diem_id_domain"></a>

## Function `remove_diem_id_domain`

Remove a DiemIdDomain from a parent VASP's DiemIdDomains resource.


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_remove_diem_id_domain">remove_diem_id_domain</a>(tc_account: &signer, <b>address</b>: <b>address</b>, domain: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_remove_diem_id_domain">remove_diem_id_domain</a>(
    tc_account: &signer,
    <b>address</b>: <b>address</b>,
    domain: vector&lt;u8&gt;,
) <b>acquires</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>, <a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>!(<a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>(), <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAIN_MANAGER">EDIEM_ID_DOMAIN_MANAGER</a>));
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAINS_NOT_PUBLISHED">EDIEM_ID_DOMAINS_NOT_PUBLISHED</a>)
    );

    <b>let</b> account_domains = <b>borrow_global_mut</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>);
    <b>let</b> diem_id_domain = <a href="DiemId.md#0x1_DiemId_create_diem_id_domain">create_diem_id_domain</a>(domain);

    <b>let</b> (<b>has</b>, index) = <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_index_of">Vector::index_of</a>(&account_domains.domains, &diem_id_domain);
    <b>if</b> (<b>has</b>) {
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_remove">Vector::remove</a>(&<b>mut</b> account_domains.domains, index);
    } <b>else</b> {
        <b>abort</b> <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemId.md#0x1_DiemId_EDOMAIN_NOT_FOUND">EDOMAIN_NOT_FOUND</a>)
    };

    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance).diem_id_domain_events,
        <a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a> {
            removed: <b>true</b>,
            domain: diem_id_domain,
            <b>address</b>: <b>address</b>,
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainAbortsIf">RemoveDiemIdDomainAbortsIf</a>;
<b>include</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainEnsures">RemoveDiemIdDomainEnsures</a>;
<b>include</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainEmits">RemoveDiemIdDomainEmits</a>;
</code></pre>




<a name="0x1_DiemId_RemoveDiemIdDomainAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainAbortsIf">RemoveDiemIdDomainAbortsIf</a> {
    tc_account: signer;
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> domains = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>).domains;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
    <b>include</b> <a href="DiemId.md#0x1_DiemId_CreateDiemIdDomainAbortsIf">CreateDiemIdDomainAbortsIf</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>) <b>with</b> Errors::NOT_PUBLISHED;
    <b>aborts_if</b> !<a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>() <b>with</b> Errors::NOT_PUBLISHED;
    <b>aborts_if</b> !contains(domains, <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain }) <b>with</b> Errors::INVALID_ARGUMENT;
}
</code></pre>




<a name="0x1_DiemId_RemoveDiemIdDomainEnsures"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainEnsures">RemoveDiemIdDomainEnsures</a> {
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> <b>post</b> domains = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(<b>address</b>).domains;
    <b>ensures</b> !contains(domains, <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain });
}
</code></pre>




<a name="0x1_DiemId_RemoveDiemIdDomainEmits"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_RemoveDiemIdDomainEmits">RemoveDiemIdDomainEmits</a> {
    tc_account: signer;
    <b>address</b>: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>let</b> handle = <b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance).diem_id_domain_events;
    <b>let</b> msg = <a href="DiemId.md#0x1_DiemId_DiemIdDomainEvent">DiemIdDomainEvent</a> {
        removed: <b>true</b>,
        domain: <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain },
        <b>address</b>,
    };
    <b>emits</b> msg <b>to</b> handle;
}
</code></pre>



</details>

<a name="0x1_DiemId_has_diem_id_domain"></a>

## Function `has_diem_id_domain`



<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_has_diem_id_domain">has_diem_id_domain</a>(addr: <b>address</b>, domain: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_has_diem_id_domain">has_diem_id_domain</a>(addr: <b>address</b>, domain: vector&lt;u8&gt;): bool <b>acquires</b> <a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a> {
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr),
        <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemId.md#0x1_DiemId_EDIEM_ID_DOMAINS_NOT_PUBLISHED">EDIEM_ID_DOMAINS_NOT_PUBLISHED</a>)
    );
    <b>let</b> account_domains = <b>borrow_global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr);
    <b>let</b> diem_id_domain = <a href="DiemId.md#0x1_DiemId_create_diem_id_domain">create_diem_id_domain</a>(domain);
    <a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(&account_domains.domains, &diem_id_domain)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemId.md#0x1_DiemId_HasDiemIdDomainAbortsIf">HasDiemIdDomainAbortsIf</a>;
<b>let</b> id_domain = <a href="DiemId.md#0x1_DiemId_DiemIdDomain">DiemIdDomain</a> { domain };
<b>ensures</b> result == contains(<b>global</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr).domains, id_domain);
</code></pre>




<a name="0x1_DiemId_HasDiemIdDomainAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemId.md#0x1_DiemId_HasDiemIdDomainAbortsIf">HasDiemIdDomainAbortsIf</a> {
    addr: <b>address</b>;
    domain: vector&lt;u8&gt;;
    <b>include</b> <a href="DiemId.md#0x1_DiemId_CreateDiemIdDomainAbortsIf">CreateDiemIdDomainAbortsIf</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomains">DiemIdDomains</a>&gt;(addr) <b>with</b> Errors::NOT_PUBLISHED;
}
</code></pre>



</details>

<a name="0x1_DiemId_tc_domain_manager_exists"></a>

## Function `tc_domain_manager_exists`



<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemId.md#0x1_DiemId_tc_domain_manager_exists">tc_domain_manager_exists</a>(): bool {
    <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="DiemId.md#0x1_DiemId_DiemIdDomainManager">DiemIdDomainManager</a>&gt;(@TreasuryCompliance);
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
