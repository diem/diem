
<a name="0x1_VASP"></a>

# Module `0x1::VASP`

### Table of Contents

-  [Resource `ParentVASP`](#0x1_VASP_ParentVASP)
-  [Resource `ChildVASP`](#0x1_VASP_ChildVASP)
-  [Resource `VASPOperationsResource`](#0x1_VASP_VASPOperationsResource)
-  [Function `initialize`](#0x1_VASP_initialize)
-  [Function `publish_parent_vasp_credential`](#0x1_VASP_publish_parent_vasp_credential)
-  [Function `publish_child_vasp_credential`](#0x1_VASP_publish_child_vasp_credential)
-  [Function `try_allow_currency`](#0x1_VASP_try_allow_currency)
-  [Function `parent_address`](#0x1_VASP_parent_address)
-  [Function `is_parent`](#0x1_VASP_is_parent)
-  [Function `is_child`](#0x1_VASP_is_child)
-  [Function `is_frozen`](#0x1_VASP_is_frozen)
-  [Function `is_vasp`](#0x1_VASP_is_vasp)
-  [Function `is_same_vasp`](#0x1_VASP_is_same_vasp)
-  [Function `num_children`](#0x1_VASP_num_children)
-  [Specification](#0x1_VASP_Specification)
    -  [Function `publish_parent_vasp_credential`](#0x1_VASP_Specification_publish_parent_vasp_credential)
    -  [Function `publish_child_vasp_credential`](#0x1_VASP_Specification_publish_child_vasp_credential)
    -  [Function `parent_address`](#0x1_VASP_Specification_parent_address)
    -  [Function `is_parent`](#0x1_VASP_Specification_is_parent)
    -  [Function `is_child`](#0x1_VASP_Specification_is_child)
    -  [Function `is_vasp`](#0x1_VASP_Specification_is_vasp)
    -  [Function `is_same_vasp`](#0x1_VASP_Specification_is_same_vasp)
    -  [Module specifications](#0x1_VASP_@Module_specifications)
    -  [Each children has a parent](#0x1_VASP_@Each_children_has_a_parent)
        -  [Privileges](#0x1_VASP_@Privileges)
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
    *num_children = *num_children + 1;
    move_to(child, <a href="#0x1_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr });
}
</code></pre>



</details>

<a name="0x1_VASP_try_allow_currency"></a>

## Function `try_allow_currency`

If the account passed in is not a VASP account, this returns true since
we don't need to ensure account limits exist for those accounts.
If the account is a child VASP account, this returns true only if a
<code>Window&lt;CoinType&gt;</code> is
published in the parent's account.
If the account is a child VASP account, this will always return true;
either a
<code>LimitsDefinition</code>/
<code>Window</code> exist for
<code>CoinType</code>, or these
will be published under the account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_try_allow_currency">try_allow_currency</a>&lt;CoinType&gt;(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_try_allow_currency">try_allow_currency</a>&lt;CoinType&gt;(account: &signer): bool
<b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a> {
    <b>assert</b>(<a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;CoinType&gt;(), ENOT_A_REGISTERED_CURRENCY);
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (!<a href="#0x1_VASP_is_vasp">is_vasp</a>(account_address)) <b>return</b> <b>true</b>;
    <b>let</b> parent_address = <a href="#0x1_VASP_parent_address">parent_address</a>(account_address);
    <b>if</b> (<a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;CoinType&gt;(parent_address)) {
        <b>true</b>
    } <b>else</b> <b>if</b> (<a href="#0x1_VASP_is_parent">is_parent</a>(account_address)) {
        <b>let</b> cap = &borrow_global&lt;<a href="#0x1_VASP_VASPOperationsResource">VASPOperationsResource</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap;
        <a href="AccountLimits.md#0x1_AccountLimits_publish_window">AccountLimits::publish_window</a>&lt;CoinType&gt;(account, cap, <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
        <b>true</b>
    } <b>else</b> {
        // it's a child vasp, and we can't publish the limits definition under it.
        <b>false</b>
    }
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

Returns true of
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

<a name="0x1_VASP_is_frozen"></a>

## Function `is_frozen`

A VASP account is frozen if itself is frozen, or if its parent account is frozen.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_frozen">is_frozen</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_is_frozen">is_frozen</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_VASP_ChildVASP">ChildVASP</a> {
    <a href="#0x1_VASP_is_vasp">is_vasp</a>(addr) && (
        <a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(<a href="#0x1_VASP_parent_address">parent_address</a>(addr)) ||
        <a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(addr)
    )
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




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role">Roles::spec_has_libra_root_role</a>(lr_account);
<b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp));
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp)) == 0;
</code></pre>



<a name="0x1_VASP_Specification_publish_child_vasp_credential"></a>

### Function `publish_child_vasp_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_parent_VASP_role">Roles::spec_has_parent_VASP_role</a>(parent);
<b>aborts_if</b> <a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child));
<b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent));
<b>aborts_if</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent)) + 1
                                    &gt; max_u64();
<b>ensures</b> <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent))
     == <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent))) + 1;
<b>ensures</b> <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child));
<b>ensures</b> TRACE(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child)))
     == TRACE(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent));
</code></pre>



<a name="0x1_VASP_Specification_parent_address"></a>

### Function `parent_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VASP_parent_address">parent_address</a>(addr: address): address
</code></pre>



TODO(wrwg): The prover hangs if we do not declare this has opaque. However, we
can still verify it (in contrast to is_child). Reason why the prover hangs
is likely related to why proving the
<code><a href="#0x1_VASP_ChildHasParent">ChildHasParent</a></code> invariant hangs.


<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> !<a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr);
<b>ensures</b> result == <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr);
</code></pre>



Spec version of
<code><a href="#0x1_VASP_parent_address">Self::parent_address</a></code>.


<a name="0x1_VASP_spec_parent_address"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr: address): address {
    <b>if</b> (<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(addr)) {
        addr
    } <b>else</b> <b>if</b> (<a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr)) {
        <b>global</b>&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    } <b>else</b> {
        0xFFFFFFFFF
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



TODO(wrwg): Because the
<code><a href="#0x1_VASP_ChildHasParent">ChildHasParent</a></code> invariant currently lets the prover hang,
we make this function opaque and specify the *expected* result. We know its true
because of the way ChildVASP is published, but can't verify this right now. This
enables verification of code which checks is_child or is_vasp.


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


TODO(wrwg): currently most global invariants make the prover hang or run very long if applied
to simple helper functions like
<code><a href="#0x1_VASP_is_vasp">Self::is_vasp</a></code>. The cause of this might be that functions
for which the invariants do not make sense (e.g. state does not change) z3 may repeatedly try
to instantiate this "dead code (dead invariants)" anyway, without getting closer to a solution.
Perhaps we may also need to generate more restricted triggers for spec lang quantifiers.
One data point seems to be that this happens only for invariants which involve the
<code><b>old</b></code>
expression. For now we have deactivated most invariants in this module, until we nail down
the problem better.

<a name="0x1_VASP_@Each_children_has_a_parent"></a>

### Each children has a parent



<a name="0x1_VASP_ChildHasParent"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_ChildHasParent">ChildHasParent</a> {
    <b>invariant</b> <b>module</b> forall a: address: <a href="#0x1_VASP_spec_child_has_parent">spec_child_has_parent</a>(a);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_ChildHasParent">ChildHasParent</a> <b>to</b> *, *&lt;CoinType&gt;;
</code></pre>


Returns true if the
<code>addr</code>, when a ChildVASP, has a ParentVASP.


<a name="0x1_VASP_spec_child_has_parent"></a>


<pre><code><b>define</b> <a href="#0x1_VASP_spec_child_has_parent">spec_child_has_parent</a>(addr: address): bool {
    <a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(addr) ==&gt; <a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<b>global</b>&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr)
}
</code></pre>



<a name="0x1_VASP_@Privileges"></a>

#### Privileges

Only a parent VASP calling publish_child_vast_credential can create
child VASP.


<a name="0x1_VASP_ChildVASPsDontChange"></a>

**Informally:** A child is at an address iff it was there in the
previous state.
TODO(wrwg): this currently lets LibraAccount hang if injected.


<pre><code><b>schema</b> <a href="#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> {
    <b>ensures</b> <b>true</b> /* forall a: address : exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a) == <b>old</b>(exists&lt;<a href="#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a)) */;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> <b>to</b> *&lt;T&gt;, * <b>except</b> publish_child_vasp_credential;
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

TODO(wrwg): this currently lets LibraAccount hang if injected.


<pre><code><b>schema</b> <a href="#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> {
    <b>ensures</b> <b>true</b> /* forall parent: address
        where <b>old</b>(<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(parent)):
            <b>old</b>(<a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent)) == <a href="#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent) */;
}
</code></pre>




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



<a name="0x1_VASP_ParentRemainsSame"></a>

TODO(wrwg): this currently lets LibraAccount hang if injected.


<pre><code><b>schema</b> <a href="#0x1_VASP_ParentRemainsSame">ParentRemainsSame</a> {
    <b>ensures</b> <b>true</b> /* forall child_addr: address
        where <b>old</b>(<a href="#0x1_VASP_spec_is_child_vasp">spec_is_child_vasp</a>(child_addr)):
            <b>old</b>(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(child_addr))
             == <a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(child_addr) */;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_ParentRemainsSame">ParentRemainsSame</a> <b>to</b> *;
</code></pre>



<a name="0x1_VASP_@Aborts_conditions_shared_between_functions."></a>

#### Aborts conditions shared between functions.



<a name="0x1_VASP_AbortsIfNotVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> {
    addr: address;
    <b>aborts_if</b> !<a href="#0x1_VASP_spec_is_vasp">spec_is_vasp</a>(addr);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> <b>to</b> parent_address, num_children;
</code></pre>




<a name="0x1_VASP_AbortsIfParentIsNotParentVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> {
    addr: address;
    <b>aborts_if</b> !<a href="#0x1_VASP_spec_is_parent_vasp">spec_is_parent_vasp</a>(<a href="#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr));
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_VASP_AbortsIfParentIsNotParentVASP">AbortsIfParentIsNotParentVASP</a> <b>to</b> num_children;
</code></pre>
