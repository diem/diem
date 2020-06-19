
<a name="SCRIPT"></a>

# Script `add_currency_to_account.move`

### Table of Contents

-  [Function `add_currency_to_account`](#SCRIPT_add_currency_to_account)



<a name="SCRIPT_add_currency_to_account"></a>

## Function `add_currency_to_account`

Add the currency identified by the type
<code>currency</code> to the sending accounts.
Aborts if the account already holds a balance fo
<code>currency</code> type.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_currency_to_account">add_currency_to_account</a>&lt;Currency&gt;(account: &signer) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;Currency&gt;(account);
}
</code></pre>



</details>
