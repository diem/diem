
<a name="SCRIPT"></a>

# Script `update_exchange_rate.move`

### Table of Contents

-  [Function `update_exchange_rate`](#SCRIPT_update_exchange_rate)



<a name="SCRIPT_update_exchange_rate"></a>

## Function `update_exchange_rate`

Update the on-chain exchange rate to LBR for the given
<code>currency</code> to be given by
<code>new_exchange_rate_numerator/new_exchange_rate_denominator</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, new_exchange_rate_numerator: u64, new_exchange_rate_denominator: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> rate = <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        new_exchange_rate_numerator,
        new_exchange_rate_denominator,
    );
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_lbr_exchange_rate">Libra::update_lbr_exchange_rate</a>&lt;Currency&gt;(tc_account, rate);
}
</code></pre>



</details>
