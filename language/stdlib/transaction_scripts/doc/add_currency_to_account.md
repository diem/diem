
<a name="add_currency_to_account"></a>

# Script `add_currency_to_account`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Adds a zero <code>Currency</code> balance to the sending <code>account</code>. This will enable <code>account</code> to
send, receive, and hold <code><a href="../../modules/doc/Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Currency&gt;</code> coins. This transaction can be
successfully sent by any account that is allowed to hold balances
(e.g., VASP, Designated Dealer).


<a name="@Technical_Description_1"></a>

## Technical Description

After the successful execution of this transaction the sending account will have a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> resource with zero balance published under it. Only
accounts that can hold balances can send this transaction, the sending account cannot
already have a <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;</code> published under it.


<a name="@Parameters_2"></a>

## Parameters

| Name       | Type      | Description                                                                                                                                         |
| ------     | ------    | -------------                                                                                                                                       |
| <code>Currency</code> | Type      | The Move type for the <code>Currency</code> being added to the sending account of the transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>account</code>  | <code>&signer</code> | The signer of the sending account of the transaction.                                                                                               |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category              | Error Reason                             | Description                                                                |
| ----------------            | --------------                           | -------------                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>     | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                  | The <code>Currency</code> is not a registered currency on-chain.                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">LibraAccount::EROLE_CANT_STORE_BALANCE</a></code> | The sending <code>account</code>'s role does not permit balances.                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">LibraAccount::EADD_EXISTING_CURRENCY</a></code>   | A balance for <code>Currency</code> is already published under the sending <code>account</code>. |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code><a href="create_child_vasp_account.md#create_child_vasp_account">Script::create_child_vasp_account</a></code>
* <code><a href="create_parent_vasp_account.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="overview.md#peer_to_peer_with_metadata">Script::peer_to_peer_with_metadata</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="add_currency_to_account.md#add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="add_currency_to_account.md#add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;Currency&gt;(account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">LibraAccount::AddCurrencyAbortsIf</a>&lt;Currency&gt;;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">LibraAccount::AddCurrencyEnsures</a>&lt;Currency&gt;;
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
</code></pre>



</details>
