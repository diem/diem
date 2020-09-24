
<a name="0x1_VASP"></a>

# Module `0x1::VASP`



-  [Resource <code><a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a></code>](#0x1_VASP_ParentVASP)
-  [Resource <code><a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a></code>](#0x1_VASP_ChildVASP)
-  [Const <code><a href="VASP.md#0x1_VASP_EPARENT_OR_CHILD_VASP">EPARENT_OR_CHILD_VASP</a></code>](#0x1_VASP_EPARENT_OR_CHILD_VASP)
-  [Const <code><a href="VASP.md#0x1_VASP_ETOO_MANY_CHILDREN">ETOO_MANY_CHILDREN</a></code>](#0x1_VASP_ETOO_MANY_CHILDREN)
-  [Const <code><a href="VASP.md#0x1_VASP_ENOT_A_VASP">ENOT_A_VASP</a></code>](#0x1_VASP_ENOT_A_VASP)
-  [Const <code><a href="VASP.md#0x1_VASP_ENOT_A_PARENT_VASP">ENOT_A_PARENT_VASP</a></code>](#0x1_VASP_ENOT_A_PARENT_VASP)
-  [Const <code><a href="VASP.md#0x1_VASP_MAX_CHILD_ACCOUNTS">MAX_CHILD_ACCOUNTS</a></code>](#0x1_VASP_MAX_CHILD_ACCOUNTS)
-  [Function <code>publish_parent_vasp_credential</code>](#0x1_VASP_publish_parent_vasp_credential)
-  [Function <code>publish_child_vasp_credential</code>](#0x1_VASP_publish_child_vasp_credential)
-  [Function <code>has_account_limits</code>](#0x1_VASP_has_account_limits)
-  [Function <code>parent_address</code>](#0x1_VASP_parent_address)
-  [Function <code>is_parent</code>](#0x1_VASP_is_parent)
-  [Function <code>is_child</code>](#0x1_VASP_is_child)
-  [Function <code>is_vasp</code>](#0x1_VASP_is_vasp)
-  [Function <code>is_same_vasp</code>](#0x1_VASP_is_same_vasp)
-  [Function <code>num_children</code>](#0x1_VASP_num_children)
-  [Module Specification](#@Module_Specification_0)
    -  [Module specifications](#@Module_specifications_1)
        -  [Existence of Parents](#@Existence_of_Parents_2)
        -  [Mutation](#@Mutation_3)
        -  [Number of children is consistent](#@Number_of_children_is_consistent_4)
        -  [Number of children does not change](#@Number_of_children_does_not_change_5)
        -  [Parent does not change](#@Parent_does_not_change_6)


<a name="0x1_VASP_ParentVASP"></a>

## Resource `ParentVASP`

Each VASP has a unique root account that holds a <code><a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a></code> resource. This resource holds
the VASP's globally unique name and all of the metadata that other VASPs need to perform
off-chain protocols with this one.


<pre><code><b>resource</b> <b>struct</b> <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>
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

A resource that represents a child account of the parent VASP account at <code>parent_vasp_addr</code>


<pre><code><b>resource</b> <b>struct</b> <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>
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

<a name="0x1_VASP_EPARENT_OR_CHILD_VASP"></a>

## Const `EPARENT_OR_CHILD_VASP`

The <code><a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a></code> or <code><a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a></code> resources are not in the required state


<pre><code><b>const</b> <a href="VASP.md#0x1_VASP_EPARENT_OR_CHILD_VASP">EPARENT_OR_CHILD_VASP</a>: u64 = 0;
</code></pre>



<a name="0x1_VASP_ETOO_MANY_CHILDREN"></a>

## Const `ETOO_MANY_CHILDREN`

The creation of a new Child VASP account would exceed the number of children permitted for a VASP


<pre><code><b>const</b> <a href="VASP.md#0x1_VASP_ETOO_MANY_CHILDREN">ETOO_MANY_CHILDREN</a>: u64 = 1;
</code></pre>



<a name="0x1_VASP_ENOT_A_VASP"></a>

## Const `ENOT_A_VASP`

The account must be a Parent or Child VASP account


<pre><code><b>const</b> <a href="VASP.md#0x1_VASP_ENOT_A_VASP">ENOT_A_VASP</a>: u64 = 2;
</code></pre>



<a name="0x1_VASP_ENOT_A_PARENT_VASP"></a>

## Const `ENOT_A_PARENT_VASP`

The creating account must be a Parent VASP account


<pre><code><b>const</b> <a href="VASP.md#0x1_VASP_ENOT_A_PARENT_VASP">ENOT_A_PARENT_VASP</a>: u64 = 3;
</code></pre>



<a name="0x1_VASP_MAX_CHILD_ACCOUNTS"></a>

## Const `MAX_CHILD_ACCOUNTS`

Maximum number of child accounts that can be created by a single ParentVASP


<pre><code><b>const</b> <a href="VASP.md#0x1_VASP_MAX_CHILD_ACCOUNTS">MAX_CHILD_ACCOUNTS</a>: u64 = 256;
</code></pre>



<a name="0x1_VASP_publish_parent_vasp_credential"></a>

## Function `publish_parent_vasp_credential`

Create a new <code><a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a></code> resource under <code>vasp</code>
Aborts if <code>lr_account</code> is not the libra root account,
or if there is already a VASP (child or parent) at this account.


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_publish_parent_vasp_credential">publish_parent_vasp_credential</a>(vasp: &signer, tc_account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="Roles.md#0x1_Roles_assert_parent_vasp_role">Roles::assert_parent_vasp_role</a>(vasp);
    <b>let</b> vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vasp);
    <b>assert</b>(!<a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(vasp_addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="VASP.md#0x1_VASP_EPARENT_OR_CHILD_VASP">EPARENT_OR_CHILD_VASP</a>));
    move_to(vasp, <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a> { num_children: 0 });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotParentVasp">Roles::AbortsIfNotParentVasp</a>{account: vasp};
<a name="0x1_VASP_vasp_addr$14"></a>
<b>let</b> vasp_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vasp);
<b>aborts_if</b> <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(vasp_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(vasp_addr);
<b>ensures</b> <a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(vasp_addr) == 0;
</code></pre>



</details>

<a name="0x1_VASP_publish_child_vasp_credential"></a>

## Function `publish_child_vasp_credential`

Create a child VASP resource for the <code>parent</code>
Aborts if <code>parent</code> is not a ParentVASP


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(parent: &signer, child: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_publish_child_vasp_credential">publish_child_vasp_credential</a>(
    parent: &signer,
    child: &signer,
) <b>acquires</b> <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a> {
    // DD: The spreadsheet does not have a "privilege" for creating
    // child VASPs. All logic in the code is based on the parent <a href="VASP.md#0x1_VASP">VASP</a> role.
    // DD: Since it checks for a <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a> property, anyway, checking
    // for role might be a bit redundant (would need <b>invariant</b> that only
    // Parent Role has <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>)
    <a href="Roles.md#0x1_Roles_assert_parent_vasp_role">Roles::assert_parent_vasp_role</a>(parent);
    <b>let</b> child_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(child);
    <b>assert</b>(!<a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(child_vasp_addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="VASP.md#0x1_VASP_EPARENT_OR_CHILD_VASP">EPARENT_OR_CHILD_VASP</a>));
    <b>let</b> parent_vasp_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(parent);
    <b>assert</b>(<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(parent_vasp_addr), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="VASP.md#0x1_VASP_ENOT_A_PARENT_VASP">ENOT_A_PARENT_VASP</a>));
    <b>let</b> num_children = &<b>mut</b> borrow_global_mut&lt;<a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent_vasp_addr).num_children;
    // Abort <b>if</b> creating this child account would put the parent <a href="VASP.md#0x1_VASP">VASP</a> over the limit
    <b>assert</b>(*<a href="VASP.md#0x1_VASP_num_children">num_children</a> &lt; <a href="VASP.md#0x1_VASP_MAX_CHILD_ACCOUNTS">MAX_CHILD_ACCOUNTS</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="VASP.md#0x1_VASP_ETOO_MANY_CHILDREN">ETOO_MANY_CHILDREN</a>));
    *num_children = *num_children + 1;
    move_to(child, <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a> { parent_vasp_addr });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_VASP_child_addr$15"></a>


<pre><code><b>let</b> child_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(child);
<b>include</b> <a href="VASP.md#0x1_VASP_PublishChildVASPAbortsIf">PublishChildVASPAbortsIf</a>{child_addr: child_addr};
<b>include</b> <a href="VASP.md#0x1_VASP_PublishChildVASPEnsures">PublishChildVASPEnsures</a>{parent_addr: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent), child_addr: child_addr};
</code></pre>




<a name="0x1_VASP_PublishChildVASPAbortsIf"></a>


<pre><code><b>schema</b> <a href="VASP.md#0x1_VASP_PublishChildVASPAbortsIf">PublishChildVASPAbortsIf</a> {
    parent: signer;
    child_addr: address;
    <a name="0x1_VASP_parent_addr$13"></a>
    <b>let</b> parent_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent);
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotParentVasp">Roles::AbortsIfNotParentVasp</a>{account: parent};
    <b>aborts_if</b> <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(child_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(parent_addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent_addr) + 1 &gt; <a href="VASP.md#0x1_VASP_MAX_CHILD_ACCOUNTS">MAX_CHILD_ACCOUNTS</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_VASP_PublishChildVASPEnsures"></a>


<pre><code><b>schema</b> <a href="VASP.md#0x1_VASP_PublishChildVASPEnsures">PublishChildVASPEnsures</a> {
    parent_addr: address;
    child_addr: address;
    <b>ensures</b> <a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent_addr) == <b>old</b>(<a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent_addr)) + 1;
    <b>ensures</b> <a href="VASP.md#0x1_VASP_is_child">is_child</a>(child_addr);
    <b>ensures</b> <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(child_addr) == parent_addr;
}
</code></pre>



</details>

<a name="0x1_VASP_has_account_limits"></a>

## Function `has_account_limits`

Return <code><b>true</b></code> if <code>addr</code> is a parent or child VASP whose parent VASP account contains an
<code><a href="AccountLimits.md#0x1_AccountLimits">AccountLimits</a>&lt;CoinType&gt;</code> resource.
Aborts if <code>addr</code> is not a VASP


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_has_account_limits">has_account_limits</a>&lt;CoinType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_has_account_limits">has_account_limits</a>&lt;CoinType&gt;(addr: address): bool <b>acquires</b> <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a> {
    <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;CoinType&gt;(<a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr))
}
</code></pre>



</details>

<a name="0x1_VASP_parent_address"></a>

## Function `parent_address`

Return <code>addr</code> if <code>addr</code> is a <code><a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a></code> or its parent's address if it is a <code><a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a></code>
Aborts otherwise


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr: address): address <b>acquires</b> <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a> {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr)) {
        addr
    } <b>else</b> <b>if</b> (<a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr)) {
        borrow_global&lt;<a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    } <b>else</b> { // wrong account type, <b>abort</b>
        <b>abort</b>(<a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="VASP.md#0x1_VASP_ENOT_A_VASP">ENOT_A_VASP</a>))
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr) && !<a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>ensures</b> result == <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr);
</code></pre>



Spec version of <code><a href="VASP.md#0x1_VASP_parent_address">Self::parent_address</a></code>.


<a name="0x1_VASP_spec_parent_address"></a>


<pre><code><b>define</b> <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr: address): address {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr)) {
        addr
    } <b>else</b> {
        <b>global</b>&lt;<a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr).parent_vasp_addr
    }
}
<a name="0x1_VASP_spec_has_account_limits"></a>
<b>define</b> <a href="VASP.md#0x1_VASP_spec_has_account_limits">spec_has_account_limits</a>&lt;Token&gt;(addr: address): bool {
    <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;Token&gt;(<a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr))
}
</code></pre>



</details>

<a name="0x1_VASP_is_parent"></a>

## Function `is_parent`

Returns true if <code>addr</code> is a parent VASP.


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr);
</code></pre>



</details>

<a name="0x1_VASP_is_child"></a>

## Function `is_child`

Returns true if <code>addr</code> is a child VASP.


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr);
</code></pre>



</details>

<a name="0x1_VASP_is_vasp"></a>

## Function `is_vasp`

Returns true if <code>addr</code> is a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr: address): bool {
    <a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(addr) || <a href="VASP.md#0x1_VASP_is_child">is_child</a>(addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr);
</code></pre>




<a name="0x1_VASP_AbortsIfNotVASP"></a>


<pre><code><b>schema</b> <a href="VASP.md#0x1_VASP_AbortsIfNotVASP">AbortsIfNotVASP</a> {
    addr: address;
    <b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr);
}
</code></pre>



</details>

<a name="0x1_VASP_is_same_vasp"></a>

## Function `is_same_vasp`

Returns true if both addresses are VASPs and they have the same parent address.


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_same_vasp">is_same_vasp</a>(addr1: address, addr2: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_is_same_vasp">is_same_vasp</a>(addr1: address, addr2: address): bool <b>acquires</b> <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a> {
    <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr1) && <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr2) && <a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr1) == <a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr2)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="VASP.md#0x1_VASP_spec_is_same_vasp">spec_is_same_vasp</a>(addr1, addr2);
</code></pre>



Spec version of <code><a href="VASP.md#0x1_VASP_is_same_vasp">Self::is_same_vasp</a></code>.


<a name="0x1_VASP_spec_is_same_vasp"></a>


<pre><code><b>define</b> <a href="VASP.md#0x1_VASP_spec_is_same_vasp">spec_is_same_vasp</a>(addr1: address, addr2: address): bool {
    <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr1) && <a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr2) && <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr1) == <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr2)
}
</code></pre>



</details>

<a name="0x1_VASP_num_children"></a>

## Function `num_children`

Return the number of child accounts for this VASP.
The total number of accounts for this VASP is num_children() + 1
Aborts if <code>addr</code> is not a ParentVASP or ChildVASP account


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_num_children">num_children</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VASP.md#0x1_VASP_num_children">num_children</a>(addr: address): u64  <b>acquires</b> <a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>, <a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a> {
    // If parent <a href="VASP.md#0x1_VASP">VASP</a> succeeds, the parent is guaranteed <b>to</b> exist.
    *&borrow_global&lt;<a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="VASP.md#0x1_VASP_parent_address">parent_address</a>(addr)).num_children
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_vasp">is_vasp</a>(addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
</code></pre>


This abort is supposed to not happen because of the parent existence invariant. However, for now
we have deactivated this invariant, so prevent this is leaking to callers with an assumed aborts
condition.


<pre><code><b>aborts_if</b> [<b>assume</b>] !<b>exists</b>&lt;<a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(<a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(addr)) <b>with</b> EXECUTION_FAILURE;
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Module_specifications_1"></a>

### Module specifications


<a name="@Existence_of_Parents_2"></a>

#### Existence of Parents



<a name="@Mutation_3"></a>

#### Mutation

Only a parent VASP calling publish_child_vast_credential can create
child VASP.


<a name="0x1_VASP_ChildVASPsDontChange"></a>

**Informally:** A child is at an address iff it was there in the
previous state.


<pre><code><b>schema</b> <a href="VASP.md#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> {
    <b>ensures</b> <b>forall</b> a: address : <b>exists</b>&lt;<a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a) == <b>old</b>(<b>exists</b>&lt;<a href="VASP.md#0x1_VASP_ChildVASP">ChildVASP</a>&gt;(a));
}
</code></pre>



TODO(wrwg): this should be replaced by a modifies clause


<pre><code><b>apply</b> <a href="VASP.md#0x1_VASP_ChildVASPsDontChange">ChildVASPsDontChange</a> <b>to</b> *&lt;T&gt;, * <b>except</b>
    publish_child_vasp_credential;
</code></pre>



<a name="@Number_of_children_is_consistent_4"></a>

#### Number of children is consistent

> PROVER TODO(emmazzz): implement the features that allows users
> to reason about number of resources with certain property,
> such as "number of ChildVASPs whose parent address is 0xDD".
> See issue #4665.

<a name="@Number_of_children_does_not_change_5"></a>

#### Number of children does not change



<a name="0x1_VASP_NumChildrenRemainsSame"></a>


<pre><code><b>schema</b> <a href="VASP.md#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> {
    <b>ensures</b> <b>forall</b> parent: address
        <b>where</b> <b>old</b>(<a href="VASP.md#0x1_VASP_is_parent">is_parent</a>(parent)):
            <b>old</b>(<a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent)) == <a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent);
}
</code></pre>



TODO(wrwg): this should be replaced by a modifies clause


<pre><code><b>apply</b> <a href="VASP.md#0x1_VASP_NumChildrenRemainsSame">NumChildrenRemainsSame</a> <b>to</b> * <b>except</b> publish_child_vasp_credential;
</code></pre>


Returns the number of children under <code>parent</code>.


<a name="0x1_VASP_spec_get_num_children"></a>


<pre><code><b>define</b> <a href="VASP.md#0x1_VASP_spec_get_num_children">spec_get_num_children</a>(parent: address): u64 {
    <b>global</b>&lt;<a href="VASP.md#0x1_VASP_ParentVASP">ParentVASP</a>&gt;(parent).num_children
}
</code></pre>



<a name="@Parent_does_not_change_6"></a>

#### Parent does not change



<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <b>forall</b> a: address <b>where</b> <b>old</b>(<a href="VASP.md#0x1_VASP_is_child">is_child</a>(a)): <a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(a) == <b>old</b>(<a href="VASP.md#0x1_VASP_spec_parent_address">spec_parent_address</a>(a));
</code></pre>
