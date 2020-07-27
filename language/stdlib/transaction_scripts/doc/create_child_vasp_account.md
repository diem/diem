
<a name="SCRIPT"></a>

# Script `create_child_vasp_account.move`

### Table of Contents

-  [Function `create_child_vasp_account`](#SCRIPT_create_child_vasp_account)
        -  [Aborts](#SCRIPT_@Aborts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `create_child_vasp_account`](#SCRIPT_Specification_create_child_vasp_account)



<a name="SCRIPT_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create a
<code>ChildVASP</code> account for sender
<code>parent_vasp</code> at
<code>child_address</code> with a balance of
<code>child_initial_balance</code> in
<code>CoinType</code> and an initial authentication_key
<code>auth_key_prefix | child_address</code>.
If
<code>add_all_currencies</code> is true, the child address will have a zero balance in all available
currencies in the system.
This account will a child of the transaction sender, which must be a ParentVASP.


<a name="SCRIPT_@Aborts"></a>

#### Aborts

The transaction will abort:

* If
<code>parent_vasp</code> is not a parent vasp with error:
<code>Roles::EINVALID_PARENT_ROLE</code>
* If
<code>child_address</code> already exists with error:
<code>Roles::EROLE_ALREADY_ASSIGNED</code>
* If
<code>parent_vasp</code> already has 256 child accounts with error:
<code>VASP::ETOO_MANY_CHILDREN</code>
* If
<code>CoinType</code> is not a registered currency with error:
<code>LibraAccount::ENOT_A_CURRENCY</code>
* If
<code>parent_vasp</code>'s withdrawal capability has been extracted with error:
<code>LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</code>
* If
<code>parent_vasp</code> doesn't hold
<code>CoinType</code> and
<code>child_initial_balance &gt; 0</code> with error:
<code>LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</code>
* If
<code>parent_vasp</code> doesn't at least
<code>child_initial_balance</code> of
<code>CoinType</code> in its account balance with error:
<code>LibraAccount::EINSUFFICIENT_BALANCE</code>


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
