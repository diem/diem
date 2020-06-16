
<a name="SCRIPT"></a>

# Script `create_designated_dealer.move`

### Table of Contents

-  [Function `create_designated_dealer`](#SCRIPT_create_designated_dealer)



<a name="SCRIPT_create_designated_dealer"></a>

## Function `create_designated_dealer`

Script for Treasury Compliance Account to create designated dealer account at 'new_account_address'
and 'auth_key_prefix' for nonsynthetic CoinType. Creates dealer and preburn resource for dd.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_designated_dealer">LibraAccount::create_designated_dealer</a>&lt;CoinType&gt;(tc_account, new_account_address, auth_key_prefix);
    // Create default tiers for newly created DD
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(tc_account, new_account_address, 500000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(tc_account, new_account_address, 5000000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(tc_account, new_account_address, 50000000);
    <a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(tc_account, new_account_address, 500000000);
}
</code></pre>



</details>
