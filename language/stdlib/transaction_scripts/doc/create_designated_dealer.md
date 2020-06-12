
<a name="SCRIPT"></a>

# Script `create_designated_dealer.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Script for Treasury Compliance Account to create designated dealer account at 'new_account_address'
and 'auth_key_prefix' for nonsynthetic CoinType. Creates dealer and preburn resource for dd.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    // XXX We need <b>to</b> figure out <b>if</b> TC is in charge of this or association root account. For now we <b>assume</b> assoc root.
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> tc_capability = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;TreasuryComplianceRole&gt;(tc_account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_designated_dealer">LibraAccount::create_designated_dealer</a>&lt;CoinType&gt;(
        tc_account,
        &tc_capability,
        new_account_address,
        auth_key_prefix
    );
    // Create default tiers for newly created DD
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(&tc_capability, new_account_address, 500000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(&tc_capability, new_account_address, 5000000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(&tc_capability, new_account_address, 50000000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(&tc_capability, new_account_address, 500000000);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, tc_capability);
}
</code></pre>



</details>
