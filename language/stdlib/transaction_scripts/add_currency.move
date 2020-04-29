script {
use 0x0::FixedPoint32;
use 0x0::Libra;

fun main<NewCurrency>(
    exchange_rate_denom: u64,
    exchange_rate_num: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector<u8>,
) {
    let rate = FixedPoint32::create_from_rational(
        exchange_rate_denom,
        exchange_rate_num,
    );
    Libra::register_currency<NewCurrency>(
        rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code,
    );
}
}
