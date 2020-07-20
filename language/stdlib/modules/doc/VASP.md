
<a name="0x1_VASP"></a>

# Module `0x1::VASP`

### Table of Contents

-  [Resource `ParentVASP`](#0x1_VASP_ParentVASP)
-  [Resource `ChildVASP`](#0x1_VASP_ChildVASP)
-  [Resource `VASPOperationsResource`](#0x1_VASP_VASPOperationsResource)
-  [Function `initialize`](#0x1_VASP_initialize)
-  [Function `publish_parent_vasp_credential`](#0x1_VASP_publish_parent_vasp_credential)
-  [Function `publish_child_vasp_credential`](#0x1_VASP_publish_child_vasp_credential)
-  [Function `has_account_limits`](#0x1_VASP_has_account_limits)
-  [Function `add_account_limits`](#0x1_VASP_add_account_limits)
-  [Function `parent_address`](#0x1_VASP_parent_address)
-  [Function `is_parent`](#0x1_VASP_is_parent)
-  [Function `is_child`](#0x1_VASP_is_child)
-  [Function `is_vasp`](#0x1_VASP_is_vasp)
-  [Function `is_same_vasp`](#0x1_VASP_is_same_vasp)
-  [Function `num_children`](#0x1_VASP_num_children)
-  [Specification](#0x1_VASP_Specification)
    -  [Function `publish_parent_vasp_credential`](#0x1_VASP_Specification_publish_parent_vasp_credential)
    -  [Function `publish_child_vasp_credential`](#0x1_VASP_Specification_publish_child_vasp_credential)
    -  [Function `has_account_limits`](#0x1_VASP_Specification_has_account_limits)
    -  [Function `parent_address`](#0x1_VASP_Specification_parent_address)
    -  [Function `is_parent`](#0x1_VASP_Specification_is_parent)
    -  [Function `is_child`](#0x1_VASP_Specification_is_child)
    -  [Function `is_vasp`](#0x1_VASP_Specification_is_vasp)
    -  [Function `is_same_vasp`](#0x1_VASP_Specification_is_same_vasp)
    -  [Module specifications](#0x1_VASP_@Module_specifications)
        -  [Post Genesis](#0x1_VASP_@Post_Genesis)
    -  [Existence of Parents](#0x1_VASP_@Existence_of_Parents)
        -  [Mutation](#0x1_VASP_@Mutation)
        -  [Number of children is consistent](#0x1_VASP_@Number_of_children_is_consistent)
        -  [Number of children does not change](#0x1_VASP_@Number_of_children_does_not_change)
        -  [Parent does not change](#0x1_VASP_@Parent_does_not_change)
        -  [Aborts conditions shared between functions.](#0x1_VASP_@Aborts_conditions_shared_between_functions.)



<a name="0x1_VASP_ParentVASP"></a>

## Resource `ParentVASP`

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

<code>num_children: u64</code>
</dt>
<dd>
 Number of child accounts this parent has created.
</dd>
</dl>


</details>

<a name="0x1_VASP_ChildVASP"></a>

## Resource `ChildVASP`

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

<a name="0x1_VASP_VASPOperationsResource"></a>

## Resource `VASPOperationsResource`

A singleton resource allowing this module to publish limits definitions and accounting windows


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_AccountLimitMutationCapability">AccountLimits::AccountLimitMutationCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VASP_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_initialize">initialize</a>(lr_account: &signer) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);
    move_to(lr_account, <a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a> {
        limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_grant_mutation_capability">AccountLimits::grant_mutation_capability</a>(lr_account),
    })
}
</code></pre>



</details>

<a name="0x1_VASP_publish_parent_vasp_credential"></a>

## Function `publish_parent_vasp_credential`

Create a new
<code><a href="#0x1_VASP_ParentVASP">ParentVASP</a></code> resource under
<code>vasp</code>
Aborts if
<code>lr_account</code> is not the libra root account,
or if there is already a VASP (child or parent) at this account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, lr_account: &signer) {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
    <b>let</b> vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vasp);
    <b>assert</b>(!<a href="#0x1_VASP_is_vasp">is_vasp</a>(vasp_addr), ENOT_A_VASP);
    move_to(vasp, <a href="#0x1_VASP_ParentVASP">ParentVASP</a> { num_children: 0 });
}
</code></pre>



</details>

<a name="0x1_VASP_publish_child_vasp_credential"></a>

## Function `publish_child_vasp_credential`

Create a child VASP resource for the
<code>parent</code>
Aborts if
<code>parent</code> is not a ParentVASP


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(
    parent: &signer,
    child: &signer,
) <b>acquires</b> <a href="#0x1_VASP_ParentVASP">ParentVASP</a> {
    // DD: The spreadsheet does not have a "privilege" for creating
    // child VASPs. All logic in the code is based on the parent <a href="#0x1_VASP">VASP</a> role.
    // DD: Since it checks for a <a href="#0x1_VASP_ParentVASP">ParentVASP</a> property, anyway, checking
    // for role might be a bit redundant (would need <b>invariant</b> that only
    // Parent Role has <a href="#0x1_VASP_ParentVASP">ParentVASP</a>)
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_parent_VASP_role">Roles::has_parent_VASP_role</a>(parent), ENOT_A_PARENT_VASP);
    <b>let</b> child_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(child);
    <b>assert</b>(!<a href="#0x1_VASP_is_vasp">is_vasp</a>(child_vasp_addr), EALREADY_A_VASP);
    <b>let</b> parent_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(parent);
    <b>assert</b>(<a href="#0x1_VASP_is_parent">is_parent</a>(parent_vasp_addr), ENOT_A_PARENT_VASP);
    <b>let</b> num_children = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_vasp_addr).num_children;
    // Abort <b>if</b> creating this child account would put the parent <a href="#0x1_VASP">VASP</a> over the limit
    <b>assert</b>(*<a href="#0x1_VASP_num_children">num_children</a> &lt; MAX_CHILD_ACCOUNTS, ETOO_MANY_CHILDREN);
    *num_children = *num_children + 1;
    move_to(child, <a href="#0x1_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr });
}
</code></pre>



</details>

<a name="0x1_VASP_has_account_limits"></a>

## Function `has_account_limits`

Return
<code><b>true</b></code> if
<code>addr</code> is a parent or child VASP whose parent VASP account contains an
<code><a href="AccountLimits.md#0x1_AccountLimits">AccountLimits</a>&lt;CoinType&gt;</code> resource.
Aborts if
<code>addr</code> is not a VASP


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_has_account_limits">has_account_limits</a>&lt;CoinType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_has_account_limits">has_account_limits</a>&lt;CoinType&gt;(addr: address): bool <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a> {
    <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;CoinType&gt;(<a href="#0x1_VASP_parent_address">parent_address</a>(addr))
}
</code></pre>



</details>

<a name="0x1_VASP_add_account_limits"></a>

## Function `add_account_limits`

Publish an
<code><a href="AccountLimits.md#0x1_AccountLimits">AccountLimits</a>&lt;CoinType&gt;</code> resource under
<code>account</code>.
Aborts if
<code>account</code> is not a ParentVASP
Aborts if
<code>account</code> already contains a
<code><a href="AccountLimits.md#0x1_AccountLimits">AccountLimits</a>&lt;CoinType&gt;</code> resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_add_account_limits">add_account_limits</a>&lt;CoinType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_add_account_limits">add_account_limits</a>&lt;CoinType&gt;(account: &signer) <b>acquires</b> <a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_parent_VASP_role">Roles::has_parent_VASP_role</a>(account), ENOT_A_PARENT_VASP);

    <a href="AccountLimits.md#0x1_AccountLimits_publish_window">AccountLimits::publish_window</a>&lt;CoinType&gt;(
        account,
        &borrow_global&lt;<a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap,
        <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()
    )
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
    <b>if</b> (<a href="#0x1_VASP_is_parent">is_parent</a>(addr)) {
        addr
    } <b>else</b> <b>if</b> (<a href="#0x1_VASP_is_child">is_child</a>(addr)) {
        borrow_global&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    } <b>else</b> { // wrong account type, <b>abort</b>
        <b>abort</b>(88)
    }
}
</code></pre>



</details>

<a name="0x1_VASP_is_parent"></a>

## Function `is_parent`

Returns true if
<code>addr</code> is a parent VASP.


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

Returns true if
<code>addr</code> is a child VASP.


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

Returns true if
<code>addr</code> is a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool {
    <a href="#0x1_VASP_is_parent">is_parent</a>(addr) || <a href="#0x1_VASP_is_child">is_child</a>(addr)
}
</code></pre>



</details>

<a name="0x1_VASP_is_same_vasp"></a>

## Function `is_same_vasp`

Returns true if both addresses are VASPs and they have the same parent address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_same_vasp">is_same_vasp</a>(addr1: address, addr2: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_same_vasp">is_same_vasp</a>(addr1: address, addr2: address): bool <b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a> {
    <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr1) && <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr2) && <a href="#0x1_VASP_parent_address">parent_address</a>(addr1) == <a href="#0x1_VASP_parent_address">parent_address</a>(addr2)
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

<a name="0x1_VASP_Specification"></a>

## Specification


<a name="0x1_VASP_Specification_publish_parent_vasp_credential"></a>

### Function `publish_parent_vasp_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, lr_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
<b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp)) == 0;
</code></pre>



<a name="0x1_VASP_Specification_publish_child_vasp_credential"></a>

### Function `publish_child_vasp_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_parent_VASP_role_addr">Roles::spec_has_parent_VASP_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent));
<b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child));
<b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent));
<b>aborts_if</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent)) + 1 &gt; 256;
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent))
     == <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent))) + 1;
<b>ensures</b> <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child));
<b>ensures</b> <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child))
     == <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent);
</code></pre>



<a name="0x1_VASP_Specification_has_account_limits"></a>

### Function `has_account_limits`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_has_account_limits">has_account_limits</a>&lt;CoinType&gt;(addr: address): bool
</code></pre>




<pre><code><b>ensures</b> result == <a href="#0x1_VASP_spec_has_account_limits">spec_has_account_limits</a>&lt;CoinType&gt;(addr);
</code></pre>




<a name="0x1_VASP_spec_has_account_limits"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_has_account_limits">spec_has_account_limits</a>&lt;CoinType&gt;(addr: address): bool {
    <a href="AccountLimits.md#0x1_AccountLimits_spec_has_window_published">AccountLimits::spec_has_window_published</a>&lt;CoinType&gt;(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr))
}
</code></pre>



<a name="0x1_VASP_Specification_parent_address"></a>

### Function `parent_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_parent_address">parent_address</a>(addr: address): address
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr);
</code></pre>



Spec version of
<code><a href="#0x1_VASP_parent_address">Self::parent_address</a></code>.


<a name="0x1_VASP_spec_parent_address"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr: address): address {
    <b>if</b> (<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr)) {
        addr
    } <b>else</b> {
        <b>global</b>&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    }
}
</code></pre>



<a name="0x1_VASP_Specification_is_parent"></a>

### Function `is_parent`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_parent">is_parent</a>(addr: address): bool
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr);
</code></pre>



Spec version of
<code><a href="#0x1_VASP_is_parent">Self::is_parent</a></code>.


<a name="0x1_VASP_spec_is_parent_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)
}
</code></pre>



<a name="0x1_VASP_Specification_is_child"></a>

### Function `is_child`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_child">is_child</a>(addr: address): bool
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr);
</code></pre>



Spec version
<code><a href="#0x1_VASP_is_child">Self::is_child</a></code>.


<a name="0x1_VASP_spec_is_child_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr: address): bool {
    exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
}
</code></pre>



<a name="0x1_VASP_Specification_is_vasp"></a>

### Function `is_vasp`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr);
</code></pre>



Spec version of
<code><a href="#0x1_VASP_is_vasp">Self::is_vasp</a></code>.


<a name="0x1_VASP_spec_is_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr: address): bool {
    <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr) || <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr)
}
</code></pre>



<a name="0x1_VASP_Specification_is_same_vasp"></a>

### Function `is_same_vasp`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_same_vasp">is_same_vasp</a>(addr1: address, addr2: address): bool
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_VASP_spec_is_same_vasp">spec_is_same_vasp</a>(addr1, addr2);
</code></pre>



Spec version of
<code><a href="#0x1_VASP_is_same_vasp">Self::is_same_vasp</a></code>.


<a name="0x1_VASP_spec_is_same_vasp"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_is_same_vasp">spec_is_same_vasp</a>(addr1: address, addr2: address): bool {
    <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr1) && <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr2) && <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr1) == <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr2)
}
</code></pre>



<a name="0x1_VASP_@Module_specifications"></a>

### Module specifications



<pre><code>pragma verify = <b>true</b>;
</code></pre>



<a name="0x1_VASP_@Post_Genesis"></a>

#### Post Genesis



<code><a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a></code> is published under the LibraRoot address after genesis.


<pre><code><b>invariant</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_up">LibraTimestamp::spec_is_up</a>() ==&gt;
        exists&lt;<a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
</code></pre>



<a name="0x1_VASP_@Existence_of_Parents"></a>

### Existence of Parents



<pre><code><b>invariant</b> [<b>global</b>]
    forall child_addr: address where <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(child_addr):
        <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<b>global</b>&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(child_addr).parent_vasp_addr);
</code></pre>



<a name="0x1_VASP_@Mutation"></a>

#### Mutation

Only a parent VASP calling publish_child_vast_credential can create
child VASP.


<a name="0x1_VASP_ChildVASPsDontChange"></a>

**Informally:** A child is at an address iff it was there in the
previous state.


<pre><code><b>schema</b> <a href="#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> {
    <b>ensures</b> forall a: address : exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a) == <b>old</b>(exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a));
}
</code></pre>



TODO(wrwg): this should be replaced by a modifies clause


<pre><code><b>apply</b> <a href="#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> <b>to</b> *&lt;T&gt;, * <b>except</b>
    publish_child_vasp_credential;
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
            <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent)) == <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent);
}
</code></pre>



TODO(wrwg): this should be replaced by a modifies clause


<pre><code><b>apply</b> <a href="#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> <b>to</b> * <b>except</b> publish_child_vasp_credential;
</code></pre>


Returns the number of children under
<code>parent</code>.


<a name="0x1_VASP_spec_get_num_children"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent: address): u64 {
    <b>global</b>&lt;<a href="#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent).num_children
}
</code></pre>



<a name="0x1_VASP_@Parent_does_not_change"></a>

#### Parent does not change



<a name="0x1_VASP_@Aborts_conditions_shared_between_functions."></a>

#### Aborts conditions shared between functions.



<a name="0x1_VASP_AbortsIfNotVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> {
    addr: address;
    <b>aborts_if</b> [export] !<a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> <b>to</b> parent_address, num_children;
</code></pre>




<a name="0x1_VASP_AbortsIfParentIsNotParentVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> {
    addr: address;
    <b>aborts_if</b> [export] !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr));
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> <b>to</b> num_children;
</code></pre>
