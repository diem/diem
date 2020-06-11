script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use 0x1::SlidingNonce;

/// Script for Treasury Comliance Account to update <Currency> to LBR rate

fun main<Currency>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_denominator: u64,
    new_exchange_rate_numerator: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let rate = FixedPoint32::create_from_rational(
        new_exchange_rate_denominator,
        new_exchange_rate_numerator,
    );
    Libra::update_lbr_exchange_rate<Currency>(tc_account, rate)
}
}
