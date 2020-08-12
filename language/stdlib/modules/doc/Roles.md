
<a name="0x1_Roles"></a>

# Module `0x1::Roles`

### Table of Contents

-  [Resource `RoleId`](#0x1_Roles_RoleId)
-  [Const `EROLE_ID`](#0x1_Roles_EROLE_ID)
-  [Const `ELIBRA_ROOT`](#0x1_Roles_ELIBRA_ROOT)
-  [Const `ETREASURY_COMPLIANCE`](#0x1_Roles_ETREASURY_COMPLIANCE)
-  [Const `EPARENT_VASP`](#0x1_Roles_EPARENT_VASP)
-  [Const `ELIBRA_ROOT_OR_TREASURY_COMPLIANCE`](#0x1_Roles_ELIBRA_ROOT_OR_TREASURY_COMPLIANCE)
-  [Const `EPARENT_VASP_OR_DESIGNATED_DEALER`](#0x1_Roles_EPARENT_VASP_OR_DESIGNATED_DEALER)
-  [Const `EDESIGNATED_DEALER`](#0x1_Roles_EDESIGNATED_DEALER)
-  [Const `EVALIDATOR`](#0x1_Roles_EVALIDATOR)
-  [Const `EVALIDATOR_OPERATOR`](#0x1_Roles_EVALIDATOR_OPERATOR)
-  [Const `LIBRA_ROOT_ROLE_ID`](#0x1_Roles_LIBRA_ROOT_ROLE_ID)
-  [Const `TREASURY_COMPLIANCE_ROLE_ID`](#0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID)
-  [Const `DESIGNATED_DEALER_ROLE_ID`](#0x1_Roles_DESIGNATED_DEALER_ROLE_ID)
-  [Const `VALIDATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_ROLE_ID)
-  [Const `VALIDATOR_OPERATOR_ROLE_ID`](#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID)
-  [Const `PARENT_VASP_ROLE_ID`](#0x1_Roles_PARENT_VASP_ROLE_ID)
-  [Const `CHILD_VASP_ROLE_ID`](#0x1_Roles_CHILD_VASP_ROLE_ID)
-  [Function `grant_libra_root_role`](#0x1_Roles_grant_libra_root_role)
-  [Function `grant_treasury_compliance_role`](#0x1_Roles_grant_treasury_compliance_role)
-  [Function `new_designated_dealer_role`](#0x1_Roles_new_designated_dealer_role)
-  [Function `new_validator_role`](#0x1_Roles_new_validator_role)
-  [Function `new_validator_operator_role`](#0x1_Roles_new_validator_operator_role)
-  [Function `new_parent_vasp_role`](#0x1_Roles_new_parent_vasp_role)
-  [Function `new_child_vasp_role`](#0x1_Roles_new_child_vasp_role)
-  [Function `grant_role`](#0x1_Roles_grant_role)
-  [Function `has_role`](#0x1_Roles_has_role)
-  [Function `has_libra_root_role`](#0x1_Roles_has_libra_root_role)
-  [Function `has_treasury_compliance_role`](#0x1_Roles_has_treasury_compliance_role)
-  [Function `has_designated_dealer_role`](#0x1_Roles_has_designated_dealer_role)
-  [Function `has_validator_role`](#0x1_Roles_has_validator_role)
-  [Function `has_validator_operator_role`](#0x1_Roles_has_validator_operator_role)
-  [Function `has_parent_VASP_role`](#0x1_Roles_has_parent_VASP_role)
-  [Function `has_child_VASP_role`](#0x1_Roles_has_child_VASP_role)
-  [Function `has_register_new_currency_privilege`](#0x1_Roles_has_register_new_currency_privilege)
-  [Function `has_update_dual_attestation_limit_privilege`](#0x1_Roles_has_update_dual_attestation_limit_privilege)
-  [Function `can_hold_balance`](#0x1_Roles_can_hold_balance)
-  [Function `needs_account_limits`](#0x1_Roles_needs_account_limits)
-  [Function `assert_libra_root`](#0x1_Roles_assert_libra_root)
-  [Function `assert_treasury_compliance`](#0x1_Roles_assert_treasury_compliance)
-  [Function `assert_parent_vasp_role`](#0x1_Roles_assert_parent_vasp_role)
-  [Function `assert_designated_dealer`](#0x1_Roles_assert_designated_dealer)
-  [Function `assert_validator`](#0x1_Roles_assert_validator)
-  [Function `assert_validator_operator`](#0x1_Roles_assert_validator_operator)
-  [Function `assert_libra_root_or_treasury_compliance`](#0x1_Roles_assert_libra_root_or_treasury_compliance)
-  [Function `assert_parent_vasp_or_designated_dealer`](#0x1_Roles_assert_parent_vasp_or_designated_dealer)
-  [Specification](#0x1_Roles_Specification)
    -  [Function `grant_libra_root_role`](#0x1_Roles_Specification_grant_libra_root_role)
    -  [Function `grant_treasury_compliance_role`](#0x1_Roles_Specification_grant_treasury_compliance_role)
    -  [Function `new_designated_dealer_role`](#0x1_Roles_Specification_new_designated_dealer_role)
    -  [Function `new_validator_role`](#0x1_Roles_Specification_new_validator_role)
    -  [Function `new_validator_operator_role`](#0x1_Roles_Specification_new_validator_operator_role)
    -  [Function `new_parent_vasp_role`](#0x1_Roles_Specification_new_parent_vasp_role)
    -  [Function `new_child_vasp_role`](#0x1_Roles_Specification_new_child_vasp_role)
    -  [Function `grant_role`](#0x1_Roles_Specification_grant_role)
    -  [Function `assert_libra_root`](#0x1_Roles_Specification_assert_libra_root)
    -  [Function `assert_treasury_compliance`](#0x1_Roles_Specification_assert_treasury_compliance)
    -  [Function `assert_parent_vasp_role`](#0x1_Roles_Specification_assert_parent_vasp_role)
    -  [Function `assert_designated_dealer`](#0x1_Roles_Specification_assert_designated_dealer)
    -  [Function `assert_validator`](#0x1_Roles_Specification_assert_validator)
    -  [Function `assert_validator_operator`](#0x1_Roles_Specification_assert_validator_operator)
    -  [Function `assert_libra_root_or_treasury_compliance`](#0x1_Roles_Specification_assert_libra_root_or_treasury_compliance)
    -  [Function `assert_parent_vasp_or_designated_dealer`](#0x1_Roles_Specification_assert_parent_vasp_or_designated_dealer)
        -  [Helper Functions and Schemas](#0x1_Roles_@Helper_Functions_and_Schemas)
        -  [Persistence of Roles](#0x1_Roles_@Persistence_of_Roles)
        -  [Conditions from Requirements](#0x1_Roles_@Conditions_from_Requirements)

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

<a name="0x1_Roles_EROLE_ID"></a>

## Const `EROLE_ID`

A
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> resource was in an unexpected state


<pre><code><b>const</b> EROLE_ID: u64 = 0;
</code></pre>



<a name="0x1_Roles_ELIBRA_ROOT"></a>

## Const `ELIBRA_ROOT`

The signer didn't have the required Libra Root role


<pre><code><b>const</b> ELIBRA_ROOT: u64 = 1;
</code></pre>



<a name="0x1_Roles_ETREASURY_COMPLIANCE"></a>

## Const `ETREASURY_COMPLIANCE`

The signer didn't have the required Treasury & Compliance role


<pre><code><b>const</b> ETREASURY_COMPLIANCE: u64 = 2;
</code></pre>



<a name="0x1_Roles_EPARENT_VASP"></a>

## Const `EPARENT_VASP`

The signer didn't have the required Parent VASP role


<pre><code><b>const</b> EPARENT_VASP: u64 = 3;
</code></pre>



<a name="0x1_Roles_ELIBRA_ROOT_OR_TREASURY_COMPLIANCE"></a>

## Const `ELIBRA_ROOT_OR_TREASURY_COMPLIANCE`

The signer didn't have the required Libra Root or Treasury & Compliance role


<pre><code><b>const</b> ELIBRA_ROOT_OR_TREASURY_COMPLIANCE: u64 = 4;
</code></pre>



<a name="0x1_Roles_EPARENT_VASP_OR_DESIGNATED_DEALER"></a>

## Const `EPARENT_VASP_OR_DESIGNATED_DEALER`

The signer didn't have the required Parent VASP or Designated Dealer role


<pre><code><b>const</b> EPARENT_VASP_OR_DESIGNATED_DEALER: u64 = 5;
</code></pre>



<a name="0x1_Roles_EDESIGNATED_DEALER"></a>

## Const `EDESIGNATED_DEALER`

The signer didn't have the required Designated Dealer role


<pre><code><b>const</b> EDESIGNATED_DEALER: u64 = 6;
</code></pre>



<a name="0x1_Roles_EVALIDATOR"></a>

## Const `EVALIDATOR`

The signer didn't have the required Validator role


<pre><code><b>const</b> EVALIDATOR: u64 = 7;
</code></pre>



<a name="0x1_Roles_EVALIDATOR_OPERATOR"></a>

## Const `EVALIDATOR_OPERATOR`

The signer didn't have the required Validator Operator role


<pre><code><b>const</b> EVALIDATOR_OPERATOR: u64 = 8;
</code></pre>



<a name="0x1_Roles_LIBRA_ROOT_ROLE_ID"></a>

## Const `LIBRA_ROOT_ROLE_ID`



<pre><code><b>const</b> LIBRA_ROOT_ROLE_ID: u64 = 0;
</code></pre>



<a name="0x1_Roles_TREASURY_COMPLIANCE_ROLE_ID"></a>

## Const `TREASURY_COMPLIANCE_ROLE_ID`



<pre><code><b>const</b> TREASURY_COMPLIANCE_ROLE_ID: u64 = 1;
</code></pre>



<a name="0x1_Roles_DESIGNATED_DEALER_ROLE_ID"></a>

## Const `DESIGNATED_DEALER_ROLE_ID`



<pre><code><b>const</b> DESIGNATED_DEALER_ROLE_ID: u64 = 2;
</code></pre>



<a name="0x1_Roles_VALIDATOR_ROLE_ID"></a>

## Const `VALIDATOR_ROLE_ID`



<pre><code><b>const</b> VALIDATOR_ROLE_ID: u64 = 3;
</code></pre>



<a name="0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID"></a>

## Const `VALIDATOR_OPERATOR_ROLE_ID`



<pre><code><b>const</b> VALIDATOR_OPERATOR_ROLE_ID: u64 = 4;
</code></pre>



<a name="0x1_Roles_PARENT_VASP_ROLE_ID"></a>

## Const `PARENT_VASP_ROLE_ID`



<pre><code><b>const</b> PARENT_VASP_ROLE_ID: u64 = 5;
</code></pre>



<a name="0x1_Roles_CHILD_VASP_ROLE_ID"></a>

## Const `CHILD_VASP_ROLE_ID`



<pre><code><b>const</b> CHILD_VASP_ROLE_ID: u64 = 6;
</code></pre>



<a name="0x1_Roles_grant_libra_root_role"></a>

## Function `grant_libra_root_role`

Granted in genesis. So there cannot be any pre-existing privileges
and roles. This is _not_ called from within LibraAccount -- these
privileges need to be created before accounts can be made
(specifically, initialization of currency)


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_libra_root_role">grant_libra_root_role</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_libra_root_role">grant_libra_root_role</a>(
    lr_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    // Grant the role <b>to</b> the libra root account
    <a href="#0x1_Roles_grant_role">grant_role</a>(lr_account, LIBRA_ROOT_ROLE_ID);
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_treasury_compliance">CoreAddresses::assert_treasury_compliance</a>(treasury_compliance_account);
    <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(lr_account);
    // Grant the TC role <b>to</b> the treasury_compliance_account
    <a href="#0x1_Roles_grant_role">grant_role</a>(treasury_compliance_account, TREASURY_COMPLIANCE_ROLE_ID);
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
    <a href="#0x1_Roles_assert_treasury_compliance">assert_treasury_compliance</a>(creating_account);
    <a href="#0x1_Roles_grant_role">grant_role</a>(new_account, DESIGNATED_DEALER_ROLE_ID);
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
    <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(creating_account);
    <a href="#0x1_Roles_grant_role">grant_role</a>(new_account, VALIDATOR_ROLE_ID);
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
    <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(creating_account);
    <a href="#0x1_Roles_grant_role">grant_role</a>(new_account, VALIDATOR_OPERATOR_ROLE_ID);
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
    // TODO(wrwg): this is implemented different than the doc. Which of them is the truth?
    <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(creating_account);
    <a href="#0x1_Roles_grant_role">grant_role</a>(new_account, PARENT_VASP_ROLE_ID);
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
    <a href="#0x1_Roles_assert_parent_vasp_role">assert_parent_vasp_role</a>(creating_account);
    <a href="#0x1_Roles_grant_role">grant_role</a>(new_account, CHILD_VASP_ROLE_ID);
}
</code></pre>



</details>

<a name="0x1_Roles_grant_role"></a>

## Function `grant_role`

Helper function to grant a role.


<pre><code><b>fun</b> <a href="#0x1_Roles_grant_role">grant_role</a>(account: &signer, role_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_grant_role">grant_role</a>(account: &signer, role_id: u64) {
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EROLE_ID));
    move_to(account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id });
}
</code></pre>



</details>

<a name="0x1_Roles_has_role"></a>

## Function `has_role`

Naming conventions: Many of the "has_*_privilege" functions do have the same body
because the spreadsheet grants all such privileges to addresses (usually a single
address) with that role. In effect, having the privilege is equivalent to having the
role, but the function names document the specific privilege involved.  Also, modules
that use these functions as a privilege check can hide specific roles, so that a change
in the privilege/role relationship can be implemented by changing Roles and not the
module that uses it.


<pre><code><b>fun</b> <a href="#0x1_Roles_has_role">has_role</a>(account: &signer, role_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Roles_has_role">has_role</a>(account: &signer, role_id: u64): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, LIBRA_ROOT_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, TREASURY_COMPLIANCE_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, DESIGNATED_DEALER_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, VALIDATOR_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, VALIDATOR_OPERATOR_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, PARENT_VASP_ROLE_ID)
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
    <a href="#0x1_Roles_has_role">has_role</a>(account, CHILD_VASP_ROLE_ID)
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
     <a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_has_update_dual_attestation_limit_privilege"></a>

## Function `has_update_dual_attestation_limit_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_update_dual_attestation_limit_privilege">has_update_dual_attestation_limit_privilege</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_has_update_dual_attestation_limit_privilege">has_update_dual_attestation_limit_privilege</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
     <a href="#0x1_Roles_has_treasury_compliance_role">has_treasury_compliance_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_can_hold_balance"></a>

## Function `can_hold_balance`

Return true if
<code>addr</code> is allowed to receive and send
<code><a href="Libra.md#0x1_Libra">Libra</a>&lt;T&gt;</code> for any T


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_can_hold_balance">can_hold_balance</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_can_hold_balance">can_hold_balance</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    // <a href="VASP.md#0x1_VASP">VASP</a> accounts and designated_dealers can hold balances.
    // Administrative accounts (`Validator`, `ValidatorOperator`, `TreasuryCompliance`, and
    // `LibraRoot`) cannot.
    <a href="#0x1_Roles_has_parent_VASP_role">has_parent_VASP_role</a>(account) ||
    <a href="#0x1_Roles_has_child_VASP_role">has_child_VASP_role</a>(account) ||
    <a href="#0x1_Roles_has_designated_dealer_role">has_designated_dealer_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_needs_account_limits"></a>

## Function `needs_account_limits`

Return true if
<code>account</code> must have limits on sending/receiving/holding of funds


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_needs_account_limits">needs_account_limits</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_needs_account_limits">needs_account_limits</a>(account: &signer): bool <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    // All accounts that hold balances are subject <b>to</b> limits <b>except</b> designated dealers
    <a href="#0x1_Roles_can_hold_balance">can_hold_balance</a>(account) && !<a href="#0x1_Roles_has_designated_dealer_role">has_designated_dealer_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_Roles_assert_libra_root"></a>

## Function `assert_libra_root`

Assert that the account is libra root.

TODO(wrwg): previously throughout the framework, we had functions which only check for the role, and
functions which check both for role and address. This is now unified via this function to always
check for both. However, the address check might be considered redundant, as we already have a global
invariant that the role of libra root and TC can only be at a specific address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(account);
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == LIBRA_ROOT_ROLE_ID, <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(ELIBRA_ROOT));
}
</code></pre>



</details>

<a name="0x1_Roles_assert_treasury_compliance"></a>

## Function `assert_treasury_compliance`

Assert that the account is treasury compliance.

TODO(wrwg): see discussion for
<code>assert_libra_root</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_treasury_compliance">CoreAddresses::assert_treasury_compliance</a>(account);
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(
        borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == TREASURY_COMPLIANCE_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(ETREASURY_COMPLIANCE)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_parent_vasp_role"></a>

## Function `assert_parent_vasp_role`

Assert that the account has the parent vasp role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_role">assert_parent_vasp_role</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_role">assert_parent_vasp_role</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(
        borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == PARENT_VASP_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(EPARENT_VASP)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_designated_dealer"></a>

## Function `assert_designated_dealer`

Assert that the account has the designated dealer role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_designated_dealer">assert_designated_dealer</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_designated_dealer">assert_designated_dealer</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(
        borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == DESIGNATED_DEALER_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(EDESIGNATED_DEALER)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_validator"></a>

## Function `assert_validator`

Assert that the account has the validator role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator">assert_validator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator">assert_validator</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(
        borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == VALIDATOR_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(EVALIDATOR)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_validator_operator"></a>

## Function `assert_validator_operator`

Assert that the account has the validator operator role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator_operator">assert_validator_operator</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator_operator">assert_validator_operator</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>assert</b>(
        borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == VALIDATOR_OPERATOR_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(EVALIDATOR_OPERATOR)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_libra_root_or_treasury_compliance"></a>

## Function `assert_libra_root_or_treasury_compliance`

Assert that the account has either the libra root or treasury compliance role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root_or_treasury_compliance">CoreAddresses::assert_libra_root_or_treasury_compliance</a>(account);
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>let</b> role_id = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
    <b>assert</b>(
        role_id == LIBRA_ROOT_ROLE_ID || role_id == TREASURY_COMPLIANCE_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(ELIBRA_ROOT_OR_TREASURY_COMPLIANCE)
    )
}
</code></pre>



</details>

<a name="0x1_Roles_assert_parent_vasp_or_designated_dealer"></a>

## Function `assert_parent_vasp_or_designated_dealer`

Assert that the account has either the parent vasp or designated dealer role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_or_designated_dealer">assert_parent_vasp_or_designated_dealer</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_or_designated_dealer">assert_parent_vasp_or_designated_dealer</a>(account: &signer) <b>acquires</b> <a href="#0x1_Roles_RoleId">RoleId</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EROLE_ID));
    <b>let</b> role_id = borrow_global&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
    <b>assert</b>(
        role_id == PARENT_VASP_ROLE_ID || role_id == DESIGNATED_DEALER_ROLE_ID,
        <a href="Errors.md#0x1_Errors_requires_role">Errors::requires_role</a>(EPARENT_VASP_OR_DESIGNATED_DEALER)
    );
}
</code></pre>



</details>

<a name="0x1_Roles_Specification"></a>

## Specification


<a name="0x1_Roles_Specification_grant_libra_root_role"></a>

### Function `grant_libra_root_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_libra_root_role">grant_libra_root_role</a>(lr_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: lr_account, role_id: LIBRA_ROOT_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_grant_treasury_compliance_role"></a>

### Function `grant_treasury_compliance_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(treasury_compliance_account: &signer, lr_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">CoreAddresses::AbortsIfNotTreasuryCompliance</a>{account: treasury_compliance_account};
<b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: treasury_compliance_account, role_id: TREASURY_COMPLIANCE_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_new_designated_dealer_role"></a>

### Function `new_designated_dealer_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_designated_dealer_role">new_designated_dealer_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>{account: creating_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: new_account, role_id: DESIGNATED_DEALER_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_new_validator_role"></a>

### Function `new_validator_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_role">new_validator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: new_account, role_id: VALIDATOR_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_new_validator_operator_role"></a>

### Function `new_validator_operator_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_operator_role">new_validator_operator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: new_account, role_id: VALIDATOR_OPERATOR_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_new_parent_vasp_role"></a>

### Function `new_parent_vasp_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_parent_vasp_role">new_parent_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: new_account, role_id: PARENT_VASP_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_new_child_vasp_role"></a>

### Function `new_child_vasp_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_child_vasp_role">new_child_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Roles_AbortsIfNotParentVasp">AbortsIfNotParentVasp</a>{account: creating_account};
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>{account: new_account, role_id: CHILD_VASP_ROLE_ID};
</code></pre>



<a name="0x1_Roles_Specification_grant_role"></a>

### Function `grant_role`


<pre><code><b>fun</b> <a href="#0x1_Roles_grant_role">grant_role</a>(account: &signer, role_id: u64)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_GrantRole">GrantRole</a>;
</code></pre>




<a name="0x1_Roles_GrantRole"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_GrantRole">GrantRole</a> {
    account: signer;
    role_id: num;
    <a name="0x1_Roles_addr$45"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>requires</b> role_id == LIBRA_ROOT_ROLE_ID ==&gt; addr == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>requires</b> role_id == TREASURY_COMPLIANCE_ROLE_ID ==&gt; addr == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
    <b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::ALREADY_PUBLISHED;
    <b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr);
    <b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == role_id;
    <b>modifies</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr);
}
</code></pre>



<a name="0x1_Roles_Specification_assert_libra_root"></a>

### Function `assert_libra_root`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root">assert_libra_root</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_treasury_compliance"></a>

### Function `assert_treasury_compliance`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_parent_vasp_role"></a>

### Function `assert_parent_vasp_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_role">assert_parent_vasp_role</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotParentVasp">AbortsIfNotParentVasp</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_designated_dealer"></a>

### Function `assert_designated_dealer`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_designated_dealer">assert_designated_dealer</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotDesignatedDealer">AbortsIfNotDesignatedDealer</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_validator"></a>

### Function `assert_validator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator">assert_validator</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotValidator">AbortsIfNotValidator</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_validator_operator"></a>

### Function `assert_validator_operator`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_validator_operator">assert_validator_operator</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotValidatorOperator">AbortsIfNotValidatorOperator</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_libra_root_or_treasury_compliance"></a>

### Function `assert_libra_root_or_treasury_compliance`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotLibraRootOrTreasuryCompliance">AbortsIfNotLibraRootOrTreasuryCompliance</a>;
</code></pre>



<a name="0x1_Roles_Specification_assert_parent_vasp_or_designated_dealer"></a>

### Function `assert_parent_vasp_or_designated_dealer`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_assert_parent_vasp_or_designated_dealer">assert_parent_vasp_or_designated_dealer</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Roles_AbortsIfNotParentVaspOrDesignatedDealer">AbortsIfNotParentVaspOrDesignatedDealer</a>;
</code></pre>




<pre><code>pragma verify = <b>true</b>;
</code></pre>



<a name="0x1_Roles_@Helper_Functions_and_Schemas"></a>

#### Helper Functions and Schemas



<a name="0x1_Roles_spec_get_role_id"></a>


<pre><code><b>define</b> <a href="#0x1_Roles_spec_get_role_id">spec_get_role_id</a>(account: signer): u64 {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id
}
<a name="0x1_Roles_spec_has_role_id_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr: address, role_id: u64): bool {
    exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) && <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == role_id
}
<a name="0x1_Roles_spec_has_libra_root_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, LIBRA_ROOT_ROLE_ID)
}
<a name="0x1_Roles_spec_has_treasury_compliance_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, TREASURY_COMPLIANCE_ROLE_ID)
}
<a name="0x1_Roles_spec_has_designated_dealer_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, DESIGNATED_DEALER_ROLE_ID)
}
<a name="0x1_Roles_spec_has_validator_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_role_addr">spec_has_validator_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, VALIDATOR_ROLE_ID)
}
<a name="0x1_Roles_spec_has_validator_operator_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_operator_role_addr">spec_has_validator_operator_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, VALIDATOR_OPERATOR_ROLE_ID)
}
<a name="0x1_Roles_spec_has_parent_VASP_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, PARENT_VASP_ROLE_ID)
}
<a name="0x1_Roles_spec_has_child_VASP_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, CHILD_VASP_ROLE_ID)
}
<a name="0x1_Roles_spec_has_register_new_currency_privilege_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_register_new_currency_privilege_addr">spec_has_register_new_currency_privilege_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr)
}
<a name="0x1_Roles_spec_has_update_dual_attestation_limit_privilege_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_update_dual_attestation_limit_privilege_addr">spec_has_update_dual_attestation_limit_privilege_addr</a>(addr: address): bool  {
    <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr)
}
<a name="0x1_Roles_spec_can_hold_balance_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr) ||
        <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr) ||
        <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr)
}
<a name="0x1_Roles_spec_needs_account_limits_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr) && !<a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr)
}
</code></pre>




<a name="0x1_Roles_ThisRoleIsNotNewlyPublished"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a> {
    this: u64;
    <b>ensures</b> forall addr: address where exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) && <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id == this:
        <b>old</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr)) && <b>old</b>(<b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id) == this;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotLibraRoot"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a> {
    account: signer;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>;
    <a name="0x1_Roles_addr$41"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != LIBRA_ROOT_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a> {
    account: signer;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">CoreAddresses::AbortsIfNotTreasuryCompliance</a>;
    <a name="0x1_Roles_addr$42"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != TREASURY_COMPLIANCE_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotLibraRootOrTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotLibraRootOrTreasuryCompliance">AbortsIfNotLibraRootOrTreasuryCompliance</a> {
    account: signer;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRootOrTreasuryCompliance">CoreAddresses::AbortsIfNotLibraRootOrTreasuryCompliance</a>;
    <a name="0x1_Roles_addr$43"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <a name="0x1_Roles_role_id$44"></a>
    <b>let</b> role_id = <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
    <b>aborts_if</b> role_id != LIBRA_ROOT_ROLE_ID && role_id != TREASURY_COMPLIANCE_ROLE_ID
        with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotParentVasp"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotParentVasp">AbortsIfNotParentVasp</a> {
    account: signer;
    <a name="0x1_Roles_addr$46"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != PARENT_VASP_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotDesignatedDealer"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotDesignatedDealer">AbortsIfNotDesignatedDealer</a> {
    account: signer;
    <a name="0x1_Roles_addr$47"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != DESIGNATED_DEALER_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotParentVaspOrDesignatedDealer"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotParentVaspOrDesignatedDealer">AbortsIfNotParentVaspOrDesignatedDealer</a> {
    account: signer;
    <a name="0x1_Roles_addr$48"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <a name="0x1_Roles_role_id$49"></a>
    <b>let</b> role_id = <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
    <b>aborts_if</b> role_id != PARENT_VASP_ROLE_ID && role_id != DESIGNATED_DEALER_ROLE_ID
        with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotValidator"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotValidator">AbortsIfNotValidator</a> {
    account: signer;
    <a name="0x1_Roles_addr$50"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != VALIDATOR_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotValidatorOperator"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotValidatorOperator">AbortsIfNotValidatorOperator</a> {
    account: signer;
    <a name="0x1_Roles_addr$51"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) with Errors::NOT_PUBLISHED;
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id != VALIDATOR_OPERATOR_ROLE_ID with Errors::REQUIRES_ROLE;
}
</code></pre>



<a name="0x1_Roles_@Persistence_of_Roles"></a>

#### Persistence of Roles

**Informally:** Once an account at address
<code>A</code> is granted a role
<code>R</code> it
will remain an account with role
<code>R</code> for all time.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    forall addr: address where <b>old</b>(exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr)):
        exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr) && <b>old</b>(<b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id) == <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id;
</code></pre>



<a name="0x1_Roles_@Conditions_from_Requirements"></a>

#### Conditions from Requirements

In this section, the conditions from the requirements for access control are systematically
applied to the functions in this module. While some of those conditions have already been
included in individual function specifications, listing them here again gives additional
assurance that that all requirements are covered.
TODO(wrwg): link to requirements

Validator roles are only granted by LibraRoot [B4]. A new
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>VALIDATOR_ROLE_ID()</code> is only
published through
<code>new_validator_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: VALIDATOR_ROLE_ID} <b>to</b> * <b>except</b> new_validator_role, grant_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_validator_role;
</code></pre>


ValidatorOperator roles are only granted by LibraRoot [B5]. A new
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>VALIDATOR_OPERATOR_ROLE_ID()</code> is only
published through
<code>new_validator_operator_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: VALIDATOR_OPERATOR_ROLE_ID} <b>to</b> * <b>except</b> new_validator_operator_role, grant_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_validator_operator_role;
</code></pre>


DesignatedDealer roles are only granted by TreasuryCompliance [B6](TODO: resolve the discrepancy). A new
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>DESIGNATED_DEALER_ROLE_ID()</code> is only
published through
<code>new_designated_dealer_role</code> which aborts if
<code>creating_account</code> does not have the TreasuryCompliance role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: DESIGNATED_DEALER_ROLE_ID} <b>to</b> * <b>except</b> new_designated_dealer_role, grant_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>{account: creating_account} <b>to</b> new_designated_dealer_role;
</code></pre>


ParentVASP roles are only granted by LibraRoot [B7]. A new
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>PARENT_VASP_ROLE_ID()</code> is only
published through
<code>new_parent_vasp_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: PARENT_VASP_ROLE_ID} <b>to</b> * <b>except</b> new_parent_vasp_role, grant_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_parent_vasp_role;
</code></pre>


ChildVASP roles are only granted by ParentVASP [B8]. A new
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>CHILD_VASP_ROLE_ID()</code> is only
published through
<code>new_child_vasp_role</code> which aborts if
<code>creating_account</code> does not have the ParentVASP role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: CHILD_VASP_ROLE_ID} <b>to</b> * <b>except</b> new_child_vasp_role, grant_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotParentVasp">AbortsIfNotParentVasp</a>{account: creating_account} <b>to</b> new_child_vasp_role;
</code></pre>


The LibraRoot role is globally unique [C2]. A
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>LIBRA_ROOT_ROLE_ID()</code> can only exists in the
<code>LIBRA_ROOT_ADDRESS()</code>. TODO: Verify that
<code>LIBRA_ROOT_ADDRESS()</code> has a LibraRoot role after
<code><a href="Genesis.md#0x1_Genesis_initialize">Genesis::initialize</a></code>.


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr):
  addr == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>


The TreasuryCompliance role is globally unique [C3]. A
<code><a href="#0x1_Roles_RoleId">RoleId</a></code> with
<code>TREASURY_COMPLIANCE_ROLE_ID()</code> can only exists in the
<code>TREASURY_COMPLIANCE_ADDRESS()</code>. TODO: Verify that
<code>TREASURY_COMPLIANCE_ADDRESS()</code> has a TreasuryCompliance role after
<code><a href="Genesis.md#0x1_Genesis_initialize">Genesis::initialize</a></code>.


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr):
  addr == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
</code></pre>


LibraRoot cannot have balances [E2].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


TreasuryCompliance cannot have balances [E3].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


Validator cannot have balances [E4].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_validator_role_addr">spec_has_validator_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ValidatorOperator cannot have balances [E5].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_validator_operator_role_addr">spec_has_validator_operator_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


DesignatedDealer have balances [E6].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ParentVASP have balances [E7].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ChildVASP have balances [E8].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


DesignatedDealer does not need account limits [F6].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


ParentVASP needs account limits [F7].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


ChildVASP needs account limits [F8].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


update_dual_attestation_limit_privilege is granted to TreasuryCompliance [B16].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_update_dual_attestation_limit_privilege_addr">spec_has_update_dual_attestation_limit_privilege_addr</a>(addr):
    <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr);
</code></pre>


register_new_currency_privilege is granted to LibraRoot [B18].


<pre><code><b>invariant</b> [<b>global</b>, on_update] forall addr: address where <a href="#0x1_Roles_spec_has_register_new_currency_privilege_addr">spec_has_register_new_currency_privilege_addr</a>(addr):
    <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr);
</code></pre>
