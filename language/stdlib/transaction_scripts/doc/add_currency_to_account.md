
<a name="SCRIPT"></a>

# Script `add_currency_to_account.move`

### Table of Contents

-  [Function `add_currency_to_account`](#SCRIPT_add_currency_to_account)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `add_currency_to_account`](#SCRIPT_Specification_add_currency_to_account)



<a name="SCRIPT_add_currency_to_account"></a>

## Function `add_currency_to_account`


<a name="SCRIPT_@Summary"></a>

### Summary

Adds a zero <code>Currency</code> balance to the sending <code>account</code>. This will enable <code>account</code> to
send, receive, and hold <code><a href="../../modules/doc/Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Currency&gt;</code> coins. This transaction can be
successfully sent by any account that is allowed to hold balances
(e.g., VASP, Designated Dealer).


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

After the successful execution of this transaction the sending account will have a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> resource with zero balance published under it. Only
accounts that can hold balances can send this transaction, the sending account cannot
already have a <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> published under it.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name       | Type      | Description                                                                                                                                         |
| ------     | ------    | -------------                                                                                                                                       |
| <code>Currency</code> | Type      | The Move type for the <code>Currency</code> being added to the sending account of the transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>account</code>  | <code>&signer</code> | The signer of the sending account of the transaction.                                                                                               |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category              | Error Reason                             | Description                                                                |
| ----------------            | --------------                           | -------------                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                  | The <code>Currency</code> is not a registered currency on-chain.                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">LibraAccount::EROLE_CANT_STORE_BALANCE</a></code> | The sending <code>account</code>'s role does not permit balances.                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">LibraAccount::EADD_EXISTING_CURRENCY</a></code>   | A balance for <code>Currency</code> is already published under the sending <code>account</code>. |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_child_vasp_account</code>
* <code>Script::create_parent_vasp_account</code>
* <code>Script::peer_to_peer_with_metadata</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;Currency&gt;(account);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_add_currency_to_account"></a>

### Function `add_currency_to_account`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer)
</code></pre>



This publishes a <code>Balance&lt;Currency&gt;</code> to the caller's account


<pre><code><b>ensures</b> exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>


<code>Currency</code> must be valid


<pre><code><b>aborts_if</b> !<a href="../../modules/doc/Libra.md#0x1_Libra_spec_is_currency">Libra::spec_is_currency</a>&lt;Currency&gt;();
</code></pre>


<code>account</code> must be allowed to hold balances


<pre><code><b>aborts_if</b> !<a href="../../modules/doc/Roles.md#0x1_Roles_spec_can_hold_balance_addr">Roles::spec_can_hold_balance_addr</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>


<code>account</code> cannot have an existing balance in <code>Currency</code>


<pre><code><b>aborts_if</b> exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>
