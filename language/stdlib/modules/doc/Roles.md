
<a name="0x1_Roles"></a>

# Module `0x1::Roles`

### Table of Contents

-  [Resource `RoleId`](#0x1_Roles_RoleId)
-  [Function `grant_libra_root_role`](#0x1_Roles_grant_libra_root_role)
-  [Function `grant_treasury_compliance_role`](#0x1_Roles_grant_treasury_compliance_role)
-  [Function `new_designated_dealer_role`](#0x1_Roles_new_designated_dealer_role)
-  [Function `new_validator_role`](#0x1_Roles_new_validator_role)
-  [Function `new_validator_operator_role`](#0x1_Roles_new_validator_operator_role)
-  [Function `new_parent_vasp_role`](#0x1_Roles_new_parent_vasp_role)
-  [Function `new_child_vasp_role`](#0x1_Roles_new_child_vasp_role)
-  [Function `has_role`](#0x1_Roles_has_role)
        -  [privilege-checking functions for roles ##](#0x1_Roles_@privilege-checking_functions_for_roles_##)
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
-  [Specification](#0x1_Roles_Specification)
    -  [Function `grant_libra_root_role`](#0x1_Roles_Specification_grant_libra_root_role)
    -  [Function `grant_treasury_compliance_role`](#0x1_Roles_Specification_grant_treasury_compliance_role)
    -  [Function `new_designated_dealer_role`](#0x1_Roles_Specification_new_designated_dealer_role)
    -  [Function `new_validator_role`](#0x1_Roles_Specification_new_validator_role)
    -  [Function `new_validator_operator_role`](#0x1_Roles_Specification_new_validator_operator_role)
    -  [Function `new_parent_vasp_role`](#0x1_Roles_Specification_new_parent_vasp_role)
    -  [Function `new_child_vasp_role`](#0x1_Roles_Specification_new_child_vasp_role)
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
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_ROOT_ADDRESS);
    // Grant the role <b>to</b> the libra root account
    move_to(lr_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: LIBRA_ROOT_ROLE_ID });
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
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(<a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(lr_account), EINVALID_PARENT_ROLE);
    <b>let</b> owner_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(treasury_compliance_account);
    <b>assert</b>(owner_address == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), EINVALID_TC_ADDRESS);
    // Grant the TC role <b>to</b> the treasury_compliance_account
    move_to(treasury_compliance_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: TREASURY_COMPLIANCE_ROLE_ID });
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
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), EROLE_ALREADY_ASSIGNED);
    <b>assert</b>(calling_role.role_id == TREASURY_COMPLIANCE_ROLE_ID, EINVALID_PARENT_ROLE);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: DESIGNATED_DEALER_ROLE_ID });
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
    <b>assert</b>(<a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(creating_account), EINVALID_PARENT_ROLE);
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), EROLE_ALREADY_ASSIGNED);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: VALIDATOR_ROLE_ID });
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
    <b>assert</b>(<a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(creating_account), EINVALID_PARENT_ROLE);
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), EROLE_ALREADY_ASSIGNED);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: VALIDATOR_OPERATOR_ROLE_ID });
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
    <b>assert</b>(<a href="#0x1_Roles_has_libra_root_role">has_libra_root_role</a>(creating_account), EINVALID_PARENT_ROLE);
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), EROLE_ALREADY_ASSIGNED);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: PARENT_VASP_ROLE_ID });
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
    <b>assert</b>(<a href="#0x1_Roles_has_parent_VASP_role">has_parent_VASP_role</a>(creating_account), EINVALID_PARENT_ROLE);
    // A role cannot have previously been assigned <b>to</b> `new_account`.
    <b>assert</b>(!exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account)), EROLE_ALREADY_ASSIGNED);
    move_to(new_account, <a href="#0x1_Roles_RoleId">RoleId</a> { role_id: CHILD_VASP_ROLE_ID });
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

<a name="0x1_Roles_Specification"></a>

## Specification


<a name="0x1_Roles_Specification_grant_libra_root_role"></a>

### Function `grant_libra_root_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_libra_root_role">grant_libra_root_role</a>(lr_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>();
<b>aborts_if</b> spec_address_of(lr_account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(lr_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(lr_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(lr_account)).role_id == <a href="#0x1_Roles_SPEC_LIBRA_ROOT_ROLE_ID">SPEC_LIBRA_ROOT_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_grant_treasury_compliance_role"></a>

### Function `grant_treasury_compliance_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_grant_treasury_compliance_role">grant_treasury_compliance_role</a>(treasury_compliance_account: &signer, lr_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>();
<b>aborts_if</b> !<a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
<b>aborts_if</b> spec_address_of(treasury_compliance_account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS</a>();
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(treasury_compliance_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(treasury_compliance_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(treasury_compliance_account)).role_id == <a href="#0x1_Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID">SPEC_TREASURY_COMPLIANCE_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_new_designated_dealer_role"></a>

### Function `new_designated_dealer_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_designated_dealer_role">new_designated_dealer_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(creating_account));
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account)).role_id == <a href="#0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID">SPEC_DESIGNATED_DEALER_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_new_validator_role"></a>

### Function `new_validator_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_role">new_validator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(creating_account));
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account)).role_id == <a href="#0x1_Roles_SPEC_VALIDATOR_ROLE_ID">SPEC_VALIDATOR_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_new_validator_operator_role"></a>

### Function `new_validator_operator_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_validator_operator_role">new_validator_operator_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(creating_account));
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account)).role_id == <a href="#0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID">SPEC_VALIDATOR_OPERATOR_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_new_parent_vasp_role"></a>

### Function `new_parent_vasp_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_parent_vasp_role">new_parent_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(creating_account));
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account)).role_id == <a href="#0x1_Roles_SPEC_PARENT_VASP_ROLE_ID">SPEC_PARENT_VASP_ROLE_ID</a>();
</code></pre>



<a name="0x1_Roles_Specification_new_child_vasp_role"></a>

### Function `new_child_vasp_role`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Roles_new_child_vasp_role">new_child_vasp_role</a>(creating_account: &signer, new_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(creating_account));
<b>aborts_if</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> exists&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(spec_address_of(new_account)).role_id == <a href="#0x1_Roles_SPEC_CHILD_VASP_ROLE_ID">SPEC_CHILD_VASP_ROLE_ID</a>();
</code></pre>


>**Note:** Just started, only a few specs.


<a name="0x1_Roles_@Role_persistence"></a>

#### Role persistence



<pre><code>pragma verify = <b>true</b>;
</code></pre>


Helper functions


<a name="0x1_Roles_spec_get_role_id"></a>


<pre><code><b>define</b> <a href="#0x1_Roles_spec_get_role_id">spec_get_role_id</a>(account: signer): u64 {
    <b>let</b> addr = spec_address_of(account);
    <b>global</b>&lt;<a href="#0x1_Roles_RoleId">RoleId</a>&gt;(addr).role_id
}
<a name="0x1_Roles_spec_has_role_id_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr: address, role_id: u64): bool {
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
<a name="0x1_Roles_spec_has_libra_root_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_LIBRA_ROOT_ROLE_ID">SPEC_LIBRA_ROOT_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_treasury_compliance_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID">SPEC_TREASURY_COMPLIANCE_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_designated_dealer_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID">SPEC_DESIGNATED_DEALER_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_validator_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_role_addr">spec_has_validator_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_VALIDATOR_ROLE_ID">SPEC_VALIDATOR_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_validator_operator_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_validator_operator_role_addr">spec_has_validator_operator_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID">SPEC_VALIDATOR_OPERATOR_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_parent_VASP_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_PARENT_VASP_ROLE_ID">SPEC_PARENT_VASP_ROLE_ID</a>())
}
<a name="0x1_Roles_spec_has_child_VASP_role_addr"></a>
<b>define</b> <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr: address): bool {
    <a href="#0x1_Roles_spec_has_role_id_addr">spec_has_role_id_addr</a>(addr, <a href="#0x1_Roles_SPEC_CHILD_VASP_ROLE_ID">SPEC_CHILD_VASP_ROLE_ID</a>())
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




<pre><code><b>apply</b> <a href="#0x1_Roles_RoleIdPersists">RoleIdPersists</a> <b>to</b> *&lt;T&gt;, * <b>except</b> has*;
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
    <b>aborts_if</b> !<a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a> {
    account: signer;
    <b>aborts_if</b> !<a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>




<a name="0x1_Roles_AbortsIfNotParentVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_Roles_AbortsIfNotParentVASP">AbortsIfNotParentVASP</a> {
    account: signer;
    <b>aborts_if</b> !<a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>



Validator roles are only granted by LibraRoot [B4]. A new
<code>RoldId</code> with
<code>VALIDATOR_ROLE_ID()</code> is only
published through
<code>new_validator_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: <a href="#0x1_Roles_SPEC_VALIDATOR_ROLE_ID">SPEC_VALIDATOR_ROLE_ID</a>()} <b>to</b> * <b>except</b> new_validator_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_validator_role;
</code></pre>


ValidatorOperator roles are only granted by LibraRoot [B5]. A new
<code>RoldId</code> with
<code>VALIDATOR_OPERATOR_ROLE_ID()</code> is only
published through
<code>new_validator_operator_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: <a href="#0x1_Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID">SPEC_VALIDATOR_OPERATOR_ROLE_ID</a>()} <b>to</b> * <b>except</b> new_validator_operator_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_validator_operator_role;
</code></pre>


DesignatedDealer roles are only granted by TreasuryCompliance [B6](TODO: resolve the discrepancy). A new
<code>RoldId</code> with
<code>DESIGNATED_DEALER_ROLE_ID()</code> is only
published through
<code>new_designated_dealer_role</code> which aborts if
<code>creating_account</code> does not have the TreasuryCompliance role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: <a href="#0x1_Roles_SPEC_DESIGNATED_DEALER_ROLE_ID">SPEC_DESIGNATED_DEALER_ROLE_ID</a>()} <b>to</b> * <b>except</b> new_designated_dealer_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>{account: creating_account} <b>to</b> new_designated_dealer_role;
</code></pre>


ParentVASP roles are only granted by LibraRoot [B7]. A new
<code>RoldId</code> with
<code>PARENT_VASP_ROLE_ID()</code> is only
published through
<code>new_parent_vasp_role</code> which aborts if
<code>creating_account</code> does not have the LibraRoot role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: <a href="#0x1_Roles_SPEC_PARENT_VASP_ROLE_ID">SPEC_PARENT_VASP_ROLE_ID</a>()} <b>to</b> * <b>except</b> new_parent_vasp_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>{account: creating_account} <b>to</b> new_parent_vasp_role;
</code></pre>


ChildVASP roles are only granted by ParentVASP [B8]. A new
<code>RoldId</code> with
<code>CHILD_VASP_ROLE_ID()</code> is only
published through
<code>new_child_vasp_role</code> which aborts if
<code>creating_account</code> does not have the ParentVASP role.


<pre><code><b>apply</b> <a href="#0x1_Roles_ThisRoleIsNotNewlyPublished">ThisRoleIsNotNewlyPublished</a>{this: <a href="#0x1_Roles_SPEC_CHILD_VASP_ROLE_ID">SPEC_CHILD_VASP_ROLE_ID</a>()} <b>to</b> * <b>except</b> new_child_vasp_role;
<b>apply</b> <a href="#0x1_Roles_AbortsIfNotParentVASP">AbortsIfNotParentVASP</a>{account: creating_account} <b>to</b> new_child_vasp_role;
</code></pre>


The LibraRoot role is globally unique [C2]. A
<code>RoldId</code> with
<code>LIBRA_ROOT_ROLE_ID()</code> can only exists in the
<code>LIBRA_ROOT_ADDRESS()</code>. TODO: Verify that
<code>LIBRA_ROOT_ADDRESS()</code> has a LibraRoot role after
<code><a href="Genesis.md#0x1_Genesis_initialize">Genesis::initialize</a></code>.


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr):
  addr == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
</code></pre>


The TreasuryCompliance role is globally unique [C3]. A
<code>RoldId</code> with
<code>TREASURY_COMPLIANCE_ROLE_ID()</code> can only exists in the
<code>TREASURY_COMPLIANCE_ADDRESS()</code>. TODO: Verify that
<code>TREASURY_COMPLIANCE_ADDRESS()</code> has a TreasuryCompliance role after
<code><a href="Genesis.md#0x1_Genesis_initialize">Genesis::initialize</a></code>.


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr):
  addr == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS</a>();
</code></pre>


LibraRoot cannot have balances [E2].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


TreasuryCompliance cannot have balances [E3].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


Validator cannot have balances [E4].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_validator_role_addr">spec_has_validator_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ValidatorOperator cannot have balances [E5].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_validator_operator_role_addr">spec_has_validator_operator_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


DesignatedDealer have balances [E6].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ParentVASP have balances [E7].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


ChildVASP have balances [E8].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_can_hold_balance_addr">spec_can_hold_balance_addr</a>(addr);
</code></pre>


DesignatedDealer does not need account limits [F6].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_designated_dealer_role_addr">spec_has_designated_dealer_role_addr</a>(addr):
    !<a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


ParentVASP needs account limits [F7].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_parent_VASP_role_addr">spec_has_parent_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


ChildVASP needs account limits [F8].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_child_VASP_role_addr">spec_has_child_VASP_role_addr</a>(addr):
    <a href="#0x1_Roles_spec_needs_account_limits_addr">spec_needs_account_limits_addr</a>(addr);
</code></pre>


update_dual_attestation_limit_privilege is granted to TreasuryCompliance [B16].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_update_dual_attestation_limit_privilege_addr">spec_has_update_dual_attestation_limit_privilege_addr</a>(addr):
    <a href="#0x1_Roles_spec_has_treasury_compliance_role_addr">spec_has_treasury_compliance_role_addr</a>(addr);
</code></pre>


register_new_currency_privilege is granted to LibraRoot [B18].


<pre><code><b>invariant</b> [<b>global</b>] forall addr: address where <a href="#0x1_Roles_spec_has_register_new_currency_privilege_addr">spec_has_register_new_currency_privilege_addr</a>(addr):
    <a href="#0x1_Roles_spec_has_libra_root_role_addr">spec_has_libra_root_role_addr</a>(addr);
</code></pre>
