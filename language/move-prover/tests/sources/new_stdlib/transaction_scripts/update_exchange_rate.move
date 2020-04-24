use 0x0::Libra;
use 0x0::FixedPoint32;
fun main<Currency>(new_exchange_rate_denominator: u64, new_exchange_rate_numerator: u64) {
    let rate = FixedPoint32::create_from_rational(
        new_exchange_rate_denominator,
        new_exchange_rate_numerator,
    );
    Libra::update_lbr_exchange_rate<Currency>(rate);
}
