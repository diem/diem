
<a name="0x1_Roles"></a>

# Module `0x1::Roles`

### Table of Contents

-  [Struct `RoleId`](#0x1_Roles_RoleId)
-  [Struct `Capability`](#0x1_Roles_Capability)
-  [Struct `Privilege`](#0x1_Roles_Privilege)
-  [Struct `AssociationRootRole`](#0x1_Roles_AssociationRootRole)
-  [Struct `TreasuryComplianceRole`](#0x1_Roles_TreasuryComplianceRole)
-  [Struct `DesignatedDealerRole`](#0x1_Roles_DesignatedDealerRole)
-  [Struct `ValidatorRole`](#0x1_Roles_ValidatorRole)
-  [Struct `ValidatorOperatorRole`](#0x1_Roles_ValidatorOperatorRole)
-  [Struct `ParentVASPRole`](#0x1_Roles_ParentVASPRole)
-  [Struct `ChildVASPRole`](#0x1_Roles_ChildVASPRole)
-  [Struct `UnhostedRole`](#0x1_Roles_UnhostedRole)
-  [Function `ASSOCIATION_ROOT_ROLE_ID`](#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID)
-  [Function `TREASURY_COMPLIANCE_ROLE_ID`](#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID)
-  [Function `DESIGNATED_DEALER_ROLE_ID`](#0x1_Roles_DESIGNATED_DEALER_ROLE_ID)
-  [Function `VALIDATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_ROLE_ID)
-  [Function `VALIDATOR_OPERATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID)
-  [Function `PARENT_VASP_ROLE_ID`](#0x1_Roles_PARENT_VASP_ROLE_ID)
-  [Function `CHILD_VASP_ROLE_ID`](#0x1_Roles_CHILD_VASP_ROLE_ID)
-  [Function `UNHOSTED_ROLE_ID`](#0x1_Roles_UNHOSTED_ROLE_ID)
-  [Function `add_privilege_to_account`](#0x1_Roles_add_privilege_to_account)
-  [Function `add_privilege_to_account_association_root_role`](#0x1_Roles_add_privilege_to_account_association_root_role)
-  [Function `add_privilege_to_account_treasury_compliance_role`](#0x1_Roles_add_privilege_to_account_treasury_compliance_role)
-  [Function `add_privilege_to_account_designated_dealer_role`](#0x1_Roles_add_privilege_to_account_designated_dealer_role)
-  [Function `add_privilege_to_account_validator_role`](#0x1_Roles_add_privilege_to_account_validator_role)
-  [Function `add_privilege_to_account_validator_operator_role`](#0x1_Roles_add_privilege_to_account_validator_operator_role)
-  [Function `add_privilege_to_account_parent_vasp_role`](#0x1_Roles_add_privilege_to_account_parent_vasp_role)
-  [Function `add_privilege_to_account_child_vasp_role`](#0x1_Roles_add_privilege_to_account_child_vasp_role)
-  [Function `add_privilege_to_account_unhosted_role`](#0x1_Roles_add_privilege_to_account_unhosted_role)
-  [Function `grant_root_association_role`](#0x1_Roles_grant_root_association_role)
-  [Function `grant_treasury_compliance_role`](#0x1_Roles_grant_treasury_compliance_role)
-  [Function `new_designated_dealer_role`](#0x1_Roles_new_designated_dealer_role)
-  [Function `new_validator_role`](#0x1_Roles_new_validator_role)
-  [Function `new_validator_operator_role`](#0x1_Roles_new_validator_operator_role)
-  [Function `new_parent_vasp_role`](#0x1_Roles_new_parent_vasp_role)
-  [Function `new_child_vasp_role`](#0x1_Roles_new_child_vasp_role)
-  [Function `new_unhosted_role`](#0x1_Roles_new_unhosted_role)
-  [Function `extract_privilege_to_capability`](#0x1_Roles_extract_privilege_to_capability)
-  [Function `restore_capability_to_privilege`](#0x1_Roles_restore_capability_to_privilege)

This module describes two things:
1. The relationship between roles, e.g. Role_A can creates accounts of Role_B
2. The granting of privileges to an account with a specific role
It is important to note here that this module _does not_ describe the
privileges that a specific role can have. This is a property of each of
the modules that declares a privilege.

It also defines functions for extracting capabilities from an
account, and ensuring that they can only be "restored" back to the
account that they were extracted from.

Roles are defined to be completely opaque outside of this module --
all operations should be guarded by privilege checks, and not by role
checks. Each role comes with a default privilege.

Terminology:
There are three main types of resources that we deal with in this
module. These are:
1. *Privilege Witnesses*
<code>P</code>: are resources that are declared in other
modules (with the exception of the default role-based privileges
defined in this module). The declaring module is responsible for
guarding the creation of resources of this type.
2. *Privileges*
<code><a href="#0x1_Roles_Privilege">Privilege</a>&lt;P&gt;</code>: where
<code>P</code> is a privilege witness is a
resource published under an account signifying that it can perform
operations that require
<code>P</code> permissions.
3. *Capabilities*
<code><a href="#0x1_Roles_Capability">Capability</a>&lt;P&gt;</code>: where
<code>P</code> is a privilege witness is
an object that represents the authority to perform actions requiring
<code>P</code> permission. These can only be extracted from accounts that hold
a
<code><a href="#0x1_Roles_Privilege">Privilege</a>&lt;P&gt;</code> resource.


<a name="0x1_Roles_RoleId"></a>

## Struct `RoleId`

The roleId contains the role id for the account. This is only moved
to an account as a top-level resource, and is otherwise immovable.
INVARIANT: Once an account at address
<code>A</code> is granted a role
<code>R</code> it
will remain an account with role
<code>R</code> for all time.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_RoleId">RoleId</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>role_id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Roles_Capability"></a>

## Struct `Capability`

Privileges are extracted in to capabilities. Capabilities hold /
the account address that they were extracted from (i.e. tagged or
"tainted"). Capabilities can then only be restored to the account
where they were extracted from.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_Capability">Capability</a>&lt;<a href="#0x1_Roles_Privilege">Privilege</a>: <b>resource</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>owner_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Roles_Privilege"></a>

## Struct `Privilege`

The internal representation of of a privilege. We wrap every
privilege witness resource here to avoid having to write extractors/restorers
for each privilege, but can instead write this generically.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv: <b>resource</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>witness: Priv</code>
</dt>
<dd>

</dd>
<dt>

<code>is_extracted: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Roles_AssociationRootRole"></a>

## Struct `AssociationRootRole`

Every role is granted a "default privilege" for that role. This can
be seen as a base-permission for every account of that role type.
INVARIANT: Every account has exactly one of these, and these
correspond precisely to the RoleId.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>
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

<a name="0x1_Roles_TreasuryComplianceRole"></a>

## Struct `TreasuryComplianceRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_TreasuryComplianceRole">TreasuryComplianceRole</a>
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

<a name="0x1_Roles_DesignatedDealerRole"></a>

## Struct `DesignatedDealerRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_DesignatedDealerRole">DesignatedDealerRole</a>
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

<a name="0x1_Roles_ValidatorRole"></a>

## Struct `ValidatorRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_ValidatorRole">ValidatorRole</a>
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

<a name="0x1_Roles_ValidatorOperatorRole"></a>

## Struct `ValidatorOperatorRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_ValidatorOperatorRole">ValidatorOperatorRole</a>
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

<a name="0x1_Roles_ParentVASPRole"></a>

## Struct `ParentVASPRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_ParentVASPRole">ParentVASPRole</a>
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

<a name="0x1_Roles_ChildVASPRole"></a>

## Struct `ChildVASPRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_ChildVASPRole">ChildVASPRole</a>
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

<a name="0x1_Roles_UnhostedRole"></a>

## Struct `UnhostedRole`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Roles_UnhostedRole">UnhostedRole</a>
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

<a name="0x1_Roles_ASSOCIATION_ROOT_ROLE_ID"></a>

## Function `ASSOCIATION_ROOT_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>(): u64 { 0 }
</code></pre>



</details>

<a name="0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID"></a>

## Function `TREASURY_COMPLIANCE_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>(): u64 { 1 }
</code></pre>



</details>

<a name="0x1_Roles_DESIGNATED_DEALER_ROLE_ID"></a>

## Function `DESIGNATED_DEALER_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>(): u64 { 2 }
</code></pre>



</details>

<a name="0x1_Roles_VALIDATOR_ROLE_ID"></a>

## Function `VALIDATOR_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>(): u64 { 3 }
</code></pre>



</details>

<a name="0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID"></a>

## Function `VALIDATOR_OPERATOR_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>(): u64 { 4 }
</code></pre>



</details>

<a name="0x1_Roles_PARENT_VASP_ROLE_ID"></a>

## Function `PARENT_VASP_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>(): u64 { 5 }
</code></pre>



</details>

<a name="0x1_Roles_CHILD_VASP_ROLE_ID"></a>

## Function `CHILD_VASP_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_CHILD_VASP_ROLE_ID">CHILD_VASP_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_CHILD_VASP_ROLE_ID">CHILD_VASP_ROLE_ID</a>(): u64 { 6 }
</code></pre>



</details>

<a name="0x1_Roles_UNHOSTED_ROLE_ID"></a>

## Function `UNHOSTED_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_UNHOSTED_ROLE_ID">UNHOSTED_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_UNHOSTED_ROLE_ID">UNHOSTED_ROLE_ID</a>(): u64 { 7 }
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account"></a>

## Function `add_privilege_to_account`

The privilege
<code>witness: Priv</code> is granted to
<code>account</code> as long as
<code>account</code> has a
<code>role</code> with
<code>role.role_id == role_id</code>.
INVARIANT: Once a privilege witness
<code>Priv</code> has been granted to an
account it remains at that account.


<pre><code><b>fun</b> <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv, role_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>&lt;Priv: <b>resource</b>&gt;(
    account: &signer,
    witness: Priv,
    role_id: u64,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> account_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>assert</b>(account_role.role_id == role_id, 0);
    move_to(account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv&gt;{ witness, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_association_root_role"></a>

## Function `add_privilege_to_account_association_root_role`

Public wrappers to the
<code>add_privilege_to_account</code> function that sets the
correct role_id for the role. This way the role that a privilege is
being assigned to outside of the module is statically determinable.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_association_root_role">add_privilege_to_account_association_root_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_association_root_role">add_privilege_to_account_association_root_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_treasury_compliance_role"></a>

## Function `add_privilege_to_account_treasury_compliance_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_treasury_compliance_role">add_privilege_to_account_treasury_compliance_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_treasury_compliance_role">add_privilege_to_account_treasury_compliance_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_designated_dealer_role"></a>

## Function `add_privilege_to_account_designated_dealer_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_designated_dealer_role">add_privilege_to_account_designated_dealer_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_designated_dealer_role">add_privilege_to_account_designated_dealer_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_validator_role"></a>

## Function `add_privilege_to_account_validator_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_validator_role">add_privilege_to_account_validator_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_validator_role">add_privilege_to_account_validator_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_validator_operator_role"></a>

## Function `add_privilege_to_account_validator_operator_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_validator_operator_role">add_privilege_to_account_validator_operator_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_validator_operator_role">add_privilege_to_account_validator_operator_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_parent_vasp_role"></a>

## Function `add_privilege_to_account_parent_vasp_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_parent_vasp_role">add_privilege_to_account_parent_vasp_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_parent_vasp_role">add_privilege_to_account_parent_vasp_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_child_vasp_role"></a>

## Function `add_privilege_to_account_child_vasp_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_child_vasp_role">add_privilege_to_account_child_vasp_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_child_vasp_role">add_privilege_to_account_child_vasp_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_CHILD_VASP_ROLE_ID">CHILD_VASP_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_add_privilege_to_account_unhosted_role"></a>

## Function `add_privilege_to_account_unhosted_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_unhosted_role">add_privilege_to_account_unhosted_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_unhosted_role">add_privilege_to_account_unhosted_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_add_privilege_to_account">add_privilege_to_account</a>(account, witness, <a href="#0x1_Roles_UNHOSTED_ROLE_ID">UNHOSTED_ROLE_ID</a>());
}
</code></pre>



</details>

<a name="0x1_Roles_grant_root_association_role"></a>

## Function `grant_root_association_role`

Granted in genesis. So there cannot be any pre-existing privileges
and roles. This is _not_ called from within LibraAccount -- these
privileges need to be created before accounts can be made
(specifically, initialization of currency)


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_root_association_role">grant_root_association_role</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_root_association_role">grant_root_association_role</a>(
    association: &signer,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), 0);
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(), 0);
    // Grant the role <b>to</b> the association root account
    move_to(association, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>() });
    move_to(association, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>&gt;{ witness: <a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>{}, is_extracted: <b>false</b>})
}
</code></pre>



</details>

<a name="0x1_Roles_grant_treasury_compliance_role"></a>

## Function `grant_treasury_compliance_role`

NB: currency-related privileges are defined in the
<code><a href="Libra.md#0x1_Libra">Libra</a></code> module.
Granted in genesis. So there cannot be any pre-existing privileges
and roles.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(treasury_compliance_account: &signer, _: &<a href="#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="#0x1_Roles_AssociationRootRole">Roles::AssociationRootRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(
    treasury_compliance_account: &signer,
    _: &<a href="#0x1_Roles_Capability">Capability</a>&lt;<a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>&gt;,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), 0);
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(treasury_compliance_account);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), 0);
    // Grant the TC role <b>to</b> the treasury_compliance_account
    move_to(treasury_compliance_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>() });
    move_to(treasury_compliance_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_TreasuryComplianceRole">TreasuryComplianceRole</a>&gt;{ witness: <a href="#0x1_Roles_TreasuryComplianceRole">TreasuryComplianceRole</a>{}, is_extracted: <b>false</b>});

    // &gt; XXX/TODO/HACK/REMOVE (tzakian): This is a _HACK_ for right now
    // so that we can allow minting <b>to</b> create an account. THIS NEEDS TO BE REMOVED.
    move_to(treasury_compliance_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>&gt;{ witness: <a href="#0x1_Roles_AssociationRootRole">AssociationRootRole</a>{}, is_extracted: <b>false</b>})
}
</code></pre>



</details>

<a name="0x1_Roles_new_designated_dealer_role"></a>

## Function `new_designated_dealer_role`

Generic new role creation (for role ids != ASSOCIATION_ROOT_ROLE_ID
and TREASURY_COMPLIANCE_ROLE_ID).
We take a
<code>&signer</code> here and link it with the account address so
that we link the
<code>signer</code> and
<code>owner_address</code> together in this
module. This should hopefully make proofs easier.

Additionally, a role comes with a default privilege for its role. This can allow
extensibility later on if a new module introduces a privilege for a role
<code>R</code>.
The new module can use a capability for the role
<code>R</code>;
<code>&<a href="#0x1_Roles_Capability">Capability</a>&lt;R&gt;</code> to guard the  granting of the privilege in order
to ensure that only those with appropriate permissions are granted
the new permission. e.g.
```
public fun publish_new_privilege(account: &signer, _: &Capability<R>) {
Roles::add_privilege_to_account(account, Roles::R_ROLE_ID());
}
``
<code>

Publish a <a href="DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a> </code>RoleId
<code> under </code>new_account
<code>.
The </code>creating_account` must be TreasuryCompliance


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_designated_dealer_role">new_designated_dealer_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_designated_dealer_role">new_designated_dealer_role</a>(
    creating_account: &signer,
    new_account: &signer,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> calling_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(creating_account));
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    //<b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>(), 0);
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>() });
    move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_DesignatedDealerRole">DesignatedDealerRole</a>&gt;{ witness: <a href="#0x1_Roles_DesignatedDealerRole">DesignatedDealerRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_new_validator_role"></a>

## Function `new_validator_role`

Publish a Validator
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.
The
<code>creating_account</code> must be LibraRoot


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_role">new_validator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_role">new_validator_role</a>(
    creating_account: &signer,
    new_account: &signer
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> calling_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(creating_account));
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>() });
    move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_ValidatorRole">ValidatorRole</a>&gt;{ witness: <a href="#0x1_Roles_ValidatorRole">ValidatorRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_new_validator_operator_role"></a>

## Function `new_validator_operator_role`

Publish a ValidatorOperator
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.
The
<code>creating_account</code> must be LibraRoot


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_operator_role">new_validator_operator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_operator_role">new_validator_operator_role</a>(
    creating_account: &signer,
    new_account: &signer,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> calling_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(creating_account));
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>() });
    move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_ValidatorOperatorRole">ValidatorOperatorRole</a>&gt;{ witness: <a href="#0x1_Roles_ValidatorOperatorRole">ValidatorOperatorRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_new_parent_vasp_role"></a>

## Function `new_parent_vasp_role`

Publish a ParentVASP
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.
The
<code>creating_account</code> must be TreasuryCompliance


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_parent_vasp_role">new_parent_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_parent_vasp_role">new_parent_vasp_role</a>(
    creating_account: &signer,
    new_account: &signer,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> calling_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(creating_account));
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    <b>assert</b>(
            calling_role.role_id == <a href="#0x1_Roles_ASSOCIATION_ROOT_ROLE_ID">ASSOCIATION_ROOT_ROLE_ID</a>()
            // XXX/HACK/REMOVE(tzakian): This is for testnet semantics
            // only. THIS NEEDS TO BE REMOVED.
            || calling_role.role_id == <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>(),
            0
        );
        move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>() });
        move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_ParentVASPRole">ParentVASPRole</a>&gt;{ witness: <a href="#0x1_Roles_ParentVASPRole">ParentVASPRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_new_child_vasp_role"></a>

## Function `new_child_vasp_role`

Publish a ChildVASP
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.
The
<code>creating_account</code> must be a ParentVASP


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_child_vasp_role">new_child_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_child_vasp_role">new_child_vasp_role</a>(
    creating_account: &signer,
    new_account: &signer,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> calling_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(creating_account));
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_CHILD_VASP_ROLE_ID">CHILD_VASP_ROLE_ID</a>() });
    move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_ChildVASPRole">ChildVASPRole</a>&gt;{ witness: <a href="#0x1_Roles_ChildVASPRole">ChildVASPRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_new_unhosted_role"></a>

## Function `new_unhosted_role`

Publish an Unhosted
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_unhosted_role">new_unhosted_role</a>(_creating_account: &signer, new_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_unhosted_role">new_unhosted_role</a>(_creating_account: &signer, new_account: &signer) {
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), 1);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_UNHOSTED_ROLE_ID">UNHOSTED_ROLE_ID</a>() });
    move_to(new_account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;<a href="#0x1_Roles_UnhostedRole">UnhostedRole</a>&gt;{ witness: <a href="#0x1_Roles_UnhostedRole">UnhostedRole</a>{}, is_extracted: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_Roles_extract_privilege_to_capability"></a>

## Function `extract_privilege_to_capability`

Some specs we may want to say about privileges and roles:
1. For all roles
<code>R = R1, ..., Rn</code> the privilege witness
<code>P</code> is only
granted to accounts with roles
<code>Ri1, Ri2, ...</code> where
<code>Rik \in R</code>.
This is a property of the module in which the privilege witness
resource
<code>P</code> is declared. (should be provable on a per-module basis)
2. For all privilege witnesses
<code>P</code>, and  instances
<code>p: Privileges&lt;P&gt;</code>, the
account at address
<code>A</code> can hold
<code>p</code> iff
<code>p.owner_address == A</code>. (should be provable)
3. Once a privilege is granted to an account
<code>A</code>, that account
holds that permission for all time.
4. Every account has one, and only one, role. The role of the
account does not change after creation.
We don't need to check for roles now, because the only way you can
get the correct capability is if you had that privilege, which can
only be granted if you have the correct role. When a capability
leaves the module we tag it with the account where it was held. You
can only put the capability back if the
<code>account</code> address you are
storing it back under and the
<code>owner_address</code> if the incoming capability agree.
INVARIANT: Once a privilege witness is created and stored under
a Privilege<PrivWitness> resource at an address A there are only two states:
1. The resource Privilege<PrivWitness> is stored at A;
2. The privilege witness is held in a Capability<PrivWitness> and
the
<code>owner_address == A</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_extract_privilege_to_capability">extract_privilege_to_capability</a>&lt;Priv: <b>resource</b>&gt;(account: &signer): <a href="#0x1_Roles_Capability">Roles::Capability</a>&lt;Priv&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_extract_privilege_to_capability">extract_privilege_to_capability</a>&lt;Priv: <b>resource</b>&gt;(account: &signer): <a href="#0x1_Roles_Capability">Capability</a>&lt;Priv&gt;
<b>acquires</b> <a href="#0x1_Roles_Privilege">Privilege</a> {
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // <a href="#0x1_Roles_Privilege">Privilege</a> doesn't exist
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv&gt;&gt;(owner_address), 3);
    <b>let</b> priv = borrow_global_mut&lt;<a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv&gt;&gt;(owner_address);
    // Make sure this privilege was not previously extracted
    <b>assert</b>(!priv.is_extracted, 4);
    // Set that the privilege is now extracted
    priv.is_extracted = <b>true</b>;
    <a href="#0x1_Roles_Capability">Capability</a>&lt;Priv&gt; { owner_address }
}
</code></pre>



</details>

<a name="0x1_Roles_restore_capability_to_privilege"></a>

## Function `restore_capability_to_privilege`

When the capability is restored back to a privilege, we make sure
that the underlying privilege cannot be stored under a different
account than it was extracted from. Once we ensure that we then
store the privilege witness back under the account.
INVARIANT: Only a capability extracted from an account A can be
restored back to A. i.e. \forall (cap: Capability<P>),
(cap.owner_address != B).  restore_capability_to_privilege<P>(B, cap) fails


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_restore_capability_to_privilege">restore_capability_to_privilege</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, cap: <a href="#0x1_Roles_Capability">Roles::Capability</a>&lt;Priv&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_restore_capability_to_privilege">restore_capability_to_privilege</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, cap: <a href="#0x1_Roles_Capability">Capability</a>&lt;Priv&gt;)
<b>acquires</b> <a href="#0x1_Roles_Privilege">Privilege</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> <a href="#0x1_Roles_Capability">Capability</a>&lt;Priv&gt;{ owner_address } = cap;
    // Make sure the owner of the privilege when we extracted it is the
    // same <b>as</b> the address we're putting it back under.
    <b>assert</b>(owner_address == account_address, 4);
    // Set that the privilege is now put back
    borrow_global_mut&lt;<a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv&gt;&gt;(owner_address).is_extracted = <b>false</b>;
}
</code></pre>



</details>
