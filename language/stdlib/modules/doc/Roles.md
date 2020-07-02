
<a name="0x1_Roles"></a>

# Module `0x1::Roles`

### Table of Contents

-  [Resource `RoleId`](#0x1_Roles_RoleId)
-  [Resource `Privilege`](#0x1_Roles_Privilege)
-  [Function `LIBRA_ROOT_ROLE_ID`](#0x1_Roles_LIBRA_ROOT_ROLE_ID)
-  [Function `TREASURY_COMPLIANCE_ROLE_ID`](#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID)
-  [Function `DESIGNATED_DEALER_ROLE_ID`](#0x1_Roles_DESIGNATED_DEALER_ROLE_ID)
-  [Function `VALIDATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_ROLE_ID)
-  [Function `VALIDATOR_OPERATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID)
-  [Function `PARENT_VASP_ROLE_ID`](#0x1_Roles_PARENT_VASP_ROLE_ID)
-  [Function `CHILD_VASP_ROLE_ID`](#0x1_Roles_CHILD_VASP_ROLE_ID)
-  [Function `UNHOSTED_ROLE_ID`](#0x1_Roles_UNHOSTED_ROLE_ID)
-  [Function `add_privilege_to_account_association_root_role`](#0x1_Roles_add_privilege_to_account_association_root_role)
-  [Function `grant_root_association_role`](#0x1_Roles_grant_root_association_role)
-  [Function `grant_treasury_compliance_role`](#0x1_Roles_grant_treasury_compliance_role)
-  [Function `new_designated_dealer_role`](#0x1_Roles_new_designated_dealer_role)
-  [Function `new_validator_role`](#0x1_Roles_new_validator_role)
-  [Function `new_validator_operator_role`](#0x1_Roles_new_validator_operator_role)
-  [Function `new_parent_vasp_role`](#0x1_Roles_new_parent_vasp_role)
-  [Function `new_child_vasp_role`](#0x1_Roles_new_child_vasp_role)
-  [Function `new_unhosted_role`](#0x1_Roles_new_unhosted_role)
-  [Function `has_role`](#0x1_Roles_has_role)
        -  [privilege-checking functions for roles ##](#0x1_Roles_@privilege-checking_functions_for_roles_##)
-  [Function `has_libra_root_role`](#0x1_Roles_has_libra_root_role)
-  [Function `has_treasury_compliance_role`](#0x1_Roles_has_treasury_compliance_role)
-  [Function `has_designated_dealer_role`](#0x1_Roles_has_designated_dealer_role)
-  [Function `has_validator_role`](#0x1_Roles_has_validator_role)
-  [Function `has_validator_operator_role`](#0x1_Roles_has_validator_operator_role)
-  [Function `has_parent_VASP_role`](#0x1_Roles_has_parent_VASP_role)
-  [Function `has_child_VASP_role`](#0x1_Roles_has_child_VASP_role)
-  [Function `has_unhosted_role`](#0x1_Roles_has_unhosted_role)
-  [Function `has_register_new_currency_privilege`](#0x1_Roles_has_register_new_currency_privilege)
-  [Function `has_update_dual_attestation_threshold_privilege`](#0x1_Roles_has_update_dual_attestation_threshold_privilege)
-  [Function `has_on_chain_config_privilege`](#0x1_Roles_has_on_chain_config_privilege)
-  [Specification](#0x1_Roles_Specification)
        -  [Role persistence](#0x1_Roles_@Role_persistence)

This module describes two things:

1. The relationship between roles, e.g. Role_A can creates accounts of Role_B
It is important to note here that this module _does not_ describe the
privileges that a specific role can have. This is a property of each of
the modules that declares a privilege.

Roles are defined to be completely opaque outside of this module --
all operations should be guarded by privilege checks, and not by role
checks. Each role comes with a default privilege.


<a name="0x1_Roles_RoleId"></a>

## Resource `RoleId`

The roleId contains the role id for the account. This is only moved
to an account as a top-level resource, and is otherwise immovable.


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

<a name="0x1_Roles_Privilege"></a>

## Resource `Privilege`

TODO: This is a legacy that will disappear soon when Tim removes a dependency
of the VM on the ModulePublish privilege.
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

<a name="0x1_Roles_LIBRA_ROOT_ROLE_ID"></a>

## Function `LIBRA_ROOT_ROLE_ID`



<pre><code><b>fun</b> <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(): u64 { 0 }
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

<a name="0x1_Roles_add_privilege_to_account_association_root_role"></a>

## Function `add_privilege_to_account_association_root_role`

TODO: This is here because the VM expects to find a ModulePublish privilege
published.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_association_root_role">add_privilege_to_account_association_root_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_add_privilege_to_account_association_root_role">add_privilege_to_account_association_root_role</a>&lt;Priv: <b>resource</b>&gt;(account: &signer, witness: Priv)
<b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> account_role = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>assert</b>(account_role.role_id == <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(), 0);
    move_to(account, <a href="#0x1_Roles_Privilege">Privilege</a>&lt;Priv&gt;{ witness, is_extracted: <b>false</b> })
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_is_genesis">LibraTimestamp::assert_is_genesis</a>();
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 0);
    // Grant the role <b>to</b> the association root account
    move_to(association, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>() });
}
</code></pre>



</details>

<a name="0x1_Roles_grant_treasury_compliance_role"></a>

## Function `grant_treasury_compliance_role`

NB: currency-related privileges are defined in the
<code><a href="Libra.md#0x1_Libra">Libra</a></code> module.
Granted in genesis. So there cannot be any pre-existing privileges
and roles.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(treasury_compliance_account: &signer, lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(
    treasury_compliance_account: &signer,
    lr_account: &signer,
) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_is_genesis">LibraTimestamp::assert_is_genesis</a>();
    <b>assert</b>(<a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(lr_account), 999);
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(treasury_compliance_account);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), 0);
    // Grant the TC role <b>to</b> the treasury_compliance_account
    move_to(treasury_compliance_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>() });
}
</code></pre>



</details>

<a name="0x1_Roles_new_designated_dealer_role"></a>

## Function `new_designated_dealer_role`

Generic new role creation (for role ids != LIBRA_ROOT_ROLE_ID
and TREASURY_COMPLIANCE_ROLE_ID).

TODO: There is some common code here that can be factored out.

Publish a DesignatedDealer
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> under
<code>new_account</code>.
The
<code>creating_account</code> must be TreasuryCompliance


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
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>() });
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
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>() });
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
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>() });
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
    <b>assert</b>(calling_role.role_id == <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>(), 0);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>() });
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
}
</code></pre>



</details>

<a name="0x1_Roles_has_role"></a>

## Function `has_role`


<a name="0x1_Roles_@privilege-checking_functions_for_roles_##"></a>

#### privilege-checking functions for roles ##


Naming conventions: Many of the "has_*_privilege" functions do have the same body
because the spreadsheet grants all such privileges to addresses (usually a single
address) with that role. In effect, having the privilege is equivalent to having the
role, but the function names document the specific privilege involved.  Also, modules
that use these functions as a privilege check can hide specific roles, so that a change
in the privilege/role relationship can be implemented by changing Roles and not the
module that uses it.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_role">has_role</a>(account: &signer, role_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_role">has_role</a>(account: &signer, role_id: u64): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
   <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
   exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr)
       && borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == role_id
}
</code></pre>



</details>

<a name="0x1_Roles_has_libra_root_role"></a>

## Function `has_libra_root_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_LIBRA_ROOT_ROLE_ID">LIBRA_ROOT_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_treasury_compliance_role"></a>

## Function `has_treasury_compliance_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_treasury_compliance_role">has_treasury_compliance_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_treasury_compliance_role">has_treasury_compliance_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID">TREASURY_COMPLIANCE_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_designated_dealer_role"></a>

## Function `has_designated_dealer_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_designated_dealer_role">has_designated_dealer_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_designated_dealer_role">has_designated_dealer_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_DESIGNATED_DEALER_ROLE_ID">DESIGNATED_DEALER_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_validator_role"></a>

## Function `has_validator_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_validator_role">has_validator_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_validator_role">has_validator_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_VALIDATOR_ROLE_ID">VALIDATOR_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_validator_operator_role"></a>

## Function `has_validator_operator_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_validator_operator_role">has_validator_operator_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_validator_operator_role">has_validator_operator_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">VALIDATOR_OPERATOR_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_parent_VASP_role"></a>

## Function `has_parent_VASP_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_parent_VASP_role">has_parent_VASP_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_parent_VASP_role">has_parent_VASP_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_PARENT_VASP_ROLE_ID">PARENT_VASP_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_child_VASP_role"></a>

## Function `has_child_VASP_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_child_VASP_role">has_child_VASP_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_child_VASP_role">has_child_VASP_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_CHILD_VASP_ROLE_ID">CHILD_VASP_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_unhosted_role"></a>

## Function `has_unhosted_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_unhosted_role">has_unhosted_role</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_unhosted_role">has_unhosted_role</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="#0x1_Roles_has_role">has_role</a>(account, <a href="#0x1_Roles_UNHOSTED_ROLE_ID">UNHOSTED_ROLE_ID</a>())
}
</code></pre>



</details>

<a name="0x1_Roles_has_register_new_currency_privilege"></a>

## Function `has_register_new_currency_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_register_new_currency_privilege">has_register_new_currency_privilege</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_register_new_currency_privilege">has_register_new_currency_privilege</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
     <a href="#0x1_Roles_has_treasury_compliance_role">has_treasury_compliance_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_has_update_dual_attestation_threshold_privilege"></a>

## Function `has_update_dual_attestation_threshold_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_update_dual_attestation_threshold_privilege">has_update_dual_attestation_threshold_privilege</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_update_dual_attestation_threshold_privilege">has_update_dual_attestation_threshold_privilege</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
     <a href="#0x1_Roles_has_treasury_compliance_role">has_treasury_compliance_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_has_on_chain_config_privilege"></a>

## Function `has_on_chain_config_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_on_chain_config_privilege">has_on_chain_config_privilege</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_on_chain_config_privilege">has_on_chain_config_privilege</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
     <a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_Specification"></a>

## Specification

>**Note:** Just started, only a few specs.


<a name="0x1_Roles_@Role_persistence"></a>

#### Role persistence



<pre><code>pragma verify = <b>true</b>;
</code></pre>


Helper functions


<a name="0x1_Roles_spec_get_role_id"></a>


<pre><code><b>define</b> <a href="#0x1_Roles_spec_get_role_id">spec_get_role_id</a>(account: signer): u64 {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id
}
<a name="0x1_Roles_spec_has_role_id"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account: signer, role_id: u64): bool {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) && <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == role_id
}
<a name="0x1_Roles_SPEC_LIBRA_ROOT_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_LIBRA_ROOT_ROLE_ID">SPEC_LIBRA_ROOT_ROLE_ID</a>(): u64 { 0 }
<a name="0x1_Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID">SPEC_TREASURY_COMPLIANCE_ROLE_ID</a>(): u64 { 1 }
<a name="0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID">SPEC_DESIGNATED_DEALER_ROLE_ID</a>(): u64 { 2 }
<a name="0x1_Roles_SPEC_VALIDATOR_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_VALIDATOR_ROLE_ID">SPEC_VALIDATOR_ROLE_ID</a>(): u64 { 3 }
<a name="0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID">SPEC_VALIDATOR_OPERATOR_ROLE_ID</a>(): u64 { 4 }
<a name="0x1_Roles_SPEC_PARENT_VASP_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_PARENT_VASP_ROLE_ID">SPEC_PARENT_VASP_ROLE_ID</a>(): u64 { 5 }
<a name="0x1_Roles_SPEC_CHILD_VASP_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_CHILD_VASP_ROLE_ID">SPEC_CHILD_VASP_ROLE_ID</a>(): u64 { 6 }
<a name="0x1_Roles_SPEC_UNHOSTED_ROLE_ID"></a>
<b>define</b> <a href="#0x1_Roles_SPEC_UNHOSTED_ROLE_ID">SPEC_UNHOSTED_ROLE_ID</a>(): u64 { 7 }
<a name="0x1_Roles_spec_has_libra_root_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_libra_root_role">spec_has_libra_root_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_LIBRA_ROOT_ROLE_ID">SPEC_LIBRA_ROOT_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_treasury_compliance_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_treasury_compliance_role">spec_has_treasury_compliance_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID">SPEC_TREASURY_COMPLIANCE_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_designated_dealer_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_designated_dealer_role">spec_has_designated_dealer_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID">SPEC_DESIGNATED_DEALER_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_validator_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_role">spec_has_validator_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_VALIDATOR_ROLE_ID">SPEC_VALIDATOR_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_validator_operator_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_operator_role">spec_has_validator_operator_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID">SPEC_VALIDATOR_OPERATOR_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_parent_VASP_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_parent_VASP_role">spec_has_parent_VASP_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_PARENT_VASP_ROLE_ID">SPEC_PARENT_VASP_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_child_VASP_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_child_VASP_role">spec_has_child_VASP_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_CHILD_VASP_ROLE_ID">SPEC_CHILD_VASP_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_unhosted_role"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_unhosted_role">spec_has_unhosted_role</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_role_id">spec_has_role_id</a>(account, <a href="#0x1_Roles_SPEC_UNHOSTED_ROLE_ID">SPEC_UNHOSTED_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_register_new_currency_privilege"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_register_new_currency_privilege">spec_has_register_new_currency_privilege</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_treasury_compliance_role">spec_has_treasury_compliance_role</a>(account)
}
<a name="0x1_Roles_spec_has_update_dual_attestation_threshold_privilege"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_update_dual_attestation_threshold_privilege">spec_has_update_dual_attestation_threshold_privilege</a>(account: signer): bool  {
    <a href="#0x1_Roles_spec_has_treasury_compliance_role">spec_has_treasury_compliance_role</a>(account)
}
<a name="0x1_Roles_spec_has_on_chain_config_privilege"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_on_chain_config_privilege">spec_has_on_chain_config_privilege</a>(account: signer): bool {
    <a href="#0x1_Roles_spec_has_treasury_compliance_role">spec_has_treasury_compliance_role</a>(account)
}
</code></pre>


**Informally:** Once an account at address
<code>A</code> is granted a role
<code>R</code> it
will remain an account with role
<code>R</code> for all time.


<a name="0x1_Roles_RoleIdPersists"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_RoleIdPersists">RoleIdPersists</a> {
    <b>ensures</b> forall addr: address where <b>old</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr)):
        exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr)
            && <b>old</b>(<b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id) == <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_Roles_RoleIdPersists">RoleIdPersists</a> <b>to</b> *&lt;T&gt;, *;
</code></pre>
