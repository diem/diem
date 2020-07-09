
<a name="SCRIPT"></a>

# Script `create_designated_dealer.move`

### Table of Contents

-  [Function `create_designated_dealer`](#SCRIPT_create_designated_dealer)



<a name="SCRIPT_create_designated_dealer"></a>

## Function `create_designated_dealer`

Create an account with the DesignatedDealer role at
<code>addr</code> with authentication key
<code>auth_key_prefix</code> |
<code>addr</code> and a 0 balance of type
<code>Currency</code>. If
<code>add_all_currencies</code> is true,
0 balances for all available currencies in the system will also be added. This can only be
invoked by an account with the TreasuryCompliance role.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_designated_dealer">create_designated_dealer</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, addr: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_designated_dealer">create_designated_dealer</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    addr: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_create_designated_dealer">LibraAccount::create_designated_dealer</a>&lt;Currency&gt;(
        tc_account,
        addr,
        auth_key_prefix,
        human_name,
        base_url,
        compliance_public_key,
        add_all_currencies
    );
}
</code></pre>



</details>
