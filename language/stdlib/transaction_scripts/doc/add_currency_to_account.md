
<a name="SCRIPT"></a>

# Script `add_currency_to_account.move`

### Table of Contents

-  [Function `add_currency_to_account`](#SCRIPT_add_currency_to_account)
-  [Specification](#SCRIPT_Specification)
    -  [Function `add_currency_to_account`](#SCRIPT_Specification_add_currency_to_account)



<a name="SCRIPT_add_currency_to_account"></a>

## Function `add_currency_to_account`

Add a
<code>Currency</code> balance to
<code>account</code>, which will enable
<code>account</code> to send and receive
<code><a href="../../modules/doc/Libra.md#0x1_Libra">Libra</a>&lt;Currency&gt;</code>.
Aborts with NOT_A_CURRENCY if
<code>Currency</code> is not an accepted currency type in the Libra system
Aborts with
<code>LibraAccount::ADD_EXISTING_CURRENCY</code> if the account already holds a balance in
<code>Currency</code>.


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




<pre><code>pragma verify = <b>true</b>;
</code></pre>


This publishes a
<code>Balance&lt;Currency&gt;</code> to the caller's account


<pre><code><b>ensures</b> exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<code>Currency</code> must be valid


<pre><code><b>aborts_if</b> !<a href="../../modules/doc/Libra.md#0x1_Libra_spec_is_currency">Libra::spec_is_currency</a>&lt;Currency&gt;();
</code></pre>



<code>account</code> must be allowed to hold balances


<pre><code><b>aborts_if</b> !<a href="../../modules/doc/Roles.md#0x1_Roles_spec_can_hold_balance_addr">Roles::spec_can_hold_balance_addr</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<code>account</code> cannot have an existing balance in
<code>Currency</code>


<pre><code><b>aborts_if</b> exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Currency&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>
