script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use 0x1::SlidingNonce;
use 0x1::Roles::{Self, TreasuryComplianceRole};

/// Update the on-chain exchange rate to LBR for the given `currency` to be given by
/// `new_exchange_rate_denominator/new_exchange_rate_numerator`.
fun update_exchange_rate<Currency>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_denominator: u64,
    new_exchange_rate_numerator: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
    let rate = FixedPoint32::create_from_rational(
        new_exchange_rate_denominator,
        new_exchange_rate_numerator,
    );
    Libra::update_lbr_exchange_rate<Currency>(&tc_capability, rate);
    Roles::restore_capability_to_privilege(tc_account, tc_capability);
}
}
