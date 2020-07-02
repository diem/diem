script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use 0x1::SlidingNonce;

/// Update the on-chain exchange rate to LBR for the given `currency` to be given by
/// `new_exchange_rate_numerator/new_exchange_rate_denominator`.
fun update_exchange_rate<Currency>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let rate = FixedPoint32::create_from_rational(
        new_exchange_rate_numerator,
        new_exchange_rate_denominator,
    );
    Libra::update_lbr_exchange_rate<Currency>(tc_account, rate);
}
}
