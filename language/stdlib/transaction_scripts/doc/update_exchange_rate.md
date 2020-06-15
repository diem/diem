
<a name="SCRIPT"></a>

# Script `update_exchange_rate.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Script for Treasury Comliance Account to update <Currency> to LBR rate


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, new_exchange_rate_denominator: u64, new_exchange_rate_numerator: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_denominator: u64,
    new_exchange_rate_numerator: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> rate = <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        new_exchange_rate_denominator,
        new_exchange_rate_numerator,
    );
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_lbr_exchange_rate">Libra::update_lbr_exchange_rate</a>&lt;Currency&gt;(tc_account, rate)
}
</code></pre>



</details>
