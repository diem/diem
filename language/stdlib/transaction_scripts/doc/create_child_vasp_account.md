
<a name="SCRIPT"></a>

# Script `create_child_vasp_account.move`

### Table of Contents

-  [Function `create_child_vasp_account`](#SCRIPT_create_child_vasp_account)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `create_child_vasp_account`](#SCRIPT_Specification_create_child_vasp_account)



<a name="SCRIPT_create_child_vasp_account"></a>

## Function `create_child_vasp_account`


<a name="SCRIPT_@Summary"></a>

### Summary

Creates a Child VASP account with its parent being the sending account of the transaction.
The sender of the transaction must be a Parent VASP account.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Creates a <code>ChildVASP</code> account for the sender <code>parent_vasp</code> at <code>child_address</code> with a balance of
<code>child_initial_balance</code> in <code>CoinType</code> and an initial authentication key of
<code>auth_key_prefix | child_address</code>.

If <code>add_all_currencies</code> is true, the child address will have a zero balance in all available
currencies in the system.

The new account will be a child account of the transaction sender, which must be a
Parent VASP account. The child account will be recorded against the limit of
child accounts of the creating Parent VASP account.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution with a <code>child_initial_balance</code> greater than zero will emit:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the <code>payer</code> field being the Parent VASP's address,
and payee field being <code>child_address</code>. This is emitted on the Parent VASP's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle.
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> with the  <code>payer</code> field being the Parent VASP's address,
and payee field being <code>child_address</code>. This is emitted on the new Child VASPS's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> handle.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                    | Type         | Description                                                                                                                                 |
| ------                  | ------       | -------------                                                                                                                               |
| <code>CoinType</code>              | Type         | The Move type for the <code>CoinType</code> that the child account should be created with. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>parent_vasp</code>           | <code>&signer</code>    | The signer reference of the sending account. Must be a Parent VASP account.                                                                 |
| <code>child_address</code>         | <code>address</code>    | Address of the to-be-created Child VASP account.                                                                                            |
| <code>auth_key_prefix</code>       | <code>vector&lt;u8&gt;</code> | The authentication key prefix that will be used initially for the newly created account.                                                    |
| <code>add_all_currencies</code>    | <code>bool</code>       | Whether to publish balance resources for all known currencies when the account is created.                                                  |
| <code>child_initial_balance</code> | <code>u64</code>        | The initial balance in <code>CoinType</code> to give the child account when it's created.                                                              |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category              | Error Reason                                             | Description                                                                              |
| ----------------            | --------------                                           | -------------                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ROLE">Errors::REQUIRES_ROLE</a></code>     | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EPARENT_VASP">Roles::EPARENT_VASP</a></code>                                    | The sending account wasn't a Parent VASP account.                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/Roles.md#0x1_Roles_EROLE_ID">Roles::EROLE_ID</a></code>                                        | The <code>child_address</code> address is already taken.                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>    | <code><a href="../../modules/doc/VASP.md#0x1_VASP_ETOO_MANY_CHILDREN">VASP::ETOO_MANY_CHILDREN</a></code>                               | The sending account has reached the maximum number of allowed child accounts.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                                  | The <code>CoinType</code> is not a registered currency on-chain.                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The withdrawal capability for the sending account has already been extracted.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | The sending account doesn't have a balance in <code>CoinType</code>.                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | The sending account doesn't have at least <code>child_initial_balance</code> of <code>CoinType</code> balance. |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_parent_vasp_account</code>
* <code>Script::add_currency</code>
* <code>Script::rotate_authentication_key</code>
* <code>Script::add_recovery_rotation_capability</code>
* <code>Script::create_recovery_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool, child_initial_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_child_vasp_account">LibraAccount::create_child_vasp_account</a>&lt;CoinType&gt;(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    <b>if</b> (child_initial_balance &gt; 0) {
        <b>let</b> vasp_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(parent_vasp);
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;CoinType&gt;(
            &vasp_withdrawal_cap, child_address, child_initial_balance, x"", x""
        );
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(vasp_withdrawal_cap);
    };
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_create_child_vasp_account"></a>

### Function `create_child_vasp_account`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_child_vasp_account">create_child_vasp_account</a>&lt;CoinType&gt;(parent_vasp: &signer, child_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool, child_initial_balance: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
pragma aborts_if_is_partial = <b>true</b>;
</code></pre>


<code>parent_vasp</code> must be a parent vasp account


<pre><code><b>aborts_if</b> !<a href="../../modules/doc/Roles.md#0x1_Roles_spec_has_parent_VASP_role_addr">Roles::spec_has_parent_VASP_role_addr</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent_vasp));
<b>aborts_if</b> !<a href="../../modules/doc/VASP.md#0x1_VASP_is_parent">VASP::is_parent</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent_vasp));
</code></pre>


<code>child_address</code> must not be an existing account/vasp account


<pre><code><b>aborts_if</b> exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(child_address);
<b>aborts_if</b> <a href="../../modules/doc/VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(child_address);
</code></pre>


<code>parent_vasp</code> must not have created more than 256 children


<pre><code><b>aborts_if</b> <a href="../../modules/doc/VASP.md#0x1_VASP_spec_get_num_children">VASP::spec_get_num_children</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent_vasp)) + 1 &gt; 256;
</code></pre>
