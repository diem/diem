address 0x1 {

module FixedPoint32 {

    /// Define a fixed-point numeric type with 32 fractional bits.
    /// This is just a u64 integer but it is wrapped in a struct to
    /// make a unique type.
    struct FixedPoint32 { value: u64 }

    const EINVALID_DIVISION: u64 = 0;

    /// Multiply a u64 integer by a fixed-point number, truncating any
    /// fractional part of the product. This will abort if the product
    /// overflows.
    public fun multiply_u64(num: u64, multiplier: FixedPoint32): u64 {
        // The product of two 64 bit values has 128 bits, so perform the
        // multiplication with u128 types and keep the full 128 bit product
        // to avoid losing accuracy.
        let unscaled_product = (num as u128) * (multiplier.value as u128);
        // The unscaled product has 32 fractional bits (from the multiplier)
        // so rescale it by shifting away the low bits.
        let product = unscaled_product >> 32;
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (product as u64)
    }
    spec fun multiply_u64 {
        /// Currently, we ignore the actual implementation of this function in verification
        /// and treat it as uninterpreted, which simplifies the verification problem significantly.
        /// This way we avoid the non-linear arithmetic problem presented by this function.
        ///
        /// Abstracting this and related functions is possible because the correctness of currency
        /// conversion (where `FixedPoint32` is used for) is not relevant for the rest of the contract
        /// control flow, so we can assume some arbitrary (but fixed) behavior here.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_multiply_u64(num, multiplier);
    }

    /// Divide a u64 integer by a fixed-point number, truncating any
    /// fractional part of the quotient. This will abort if the divisor
    /// is zero or if the quotient overflows.
    public fun divide_u64(num: u64, divisor: FixedPoint32): u64 {
        // First convert to 128 bits and then shift left to
        // add 32 fractional zero bits to the dividend.
        let scaled_value = (num as u128) << 32;
        // this will fail with a divide-by-zero error.
        let quotient = scaled_value / (divisor.value as u128);
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (quotient as u64)
    }
    spec fun divide_u64 {
        /// See comment at `Self::multiply_64`.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_divide_u64(num, divisor);
    }

    /// Create a fixed-point value from a rational number specified by its
    /// numerator and denominator. This function is for convenience; it is also
    /// perfectly fine to create a fixed-point value by directly specifying the
    /// raw value. This will abort if the denominator is zero or if the ratio is
    /// not in the range 2^-32 .. 2^32-1.
    public fun create_from_rational(numerator: u64, denominator: u64): FixedPoint32 {
        // Scale the numerator to have 64 fractional bits and the denominator
        // to have 32 fractional bits, so that the quotient will have 32
        // fractional bits.
        let scaled_numerator = (numerator as u128) << 64;
        let scaled_denominator = (denominator as u128) << 32;
        // If the denominator is zero, this will fail with a divide-by-zero
        // error.
        let quotient = scaled_numerator / scaled_denominator;
        // but if you really want a ratio of zero, it is easy to create that
        // from a raw value.
        assert(quotient != 0 || numerator == 0, EINVALID_DIVISION);
        // Return the quotient as a fixed-point number. The cast will fail
        // with an arithmetic error if the number is too large.
        FixedPoint32 { value: (quotient as u64) }
    }
    spec fun create_from_rational {
        /// See comment at `Self::multiply_64`.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_create_from_rational(numerator, denominator);
    }

    public fun create_from_raw_value(value: u64): FixedPoint32 {
        FixedPoint32 { value }
    }

    /// Accessor for the raw u64 value. Other less common operations, such as
    /// adding or subtracting FixedPoint32 values, can be done using the raw
    /// values directly.
    public fun get_raw_value(num: FixedPoint32): u64 {
        num.value
    }

    // **************** SPECIFICATIONS ****************

    spec module {
        /// Uninterpreted function for `Self::multiply_u64`.
        define spec_multiply_u64(val: u64, multiplier: FixedPoint32): u64;

        /// Uninterpreted function for `Self::divide_u64`.
        define spec_divide_u64(val: u64, divisor: FixedPoint32): u64;

        /// Uninterpreted function for `Self::create_from_rational`.
        define spec_create_from_rational(numerator: u64, denominator: u64): FixedPoint32;
    }


    // TODO(wrwg): the below should be reviewed and perhaps revised/removed, it is not clear
    // how helpful it is.

    /*
    This module defines the fixed-point numeric type, the constructors and the operations (mul, div) for that type.
    It is unlikely that this module would have any module invariant (or global property) because it does not declare
    any resource type whose value can be stored and retained in the global state. However, it is possible that
    FixedPoint32::T is used in the invariants of other modules.

    An apparent local property to check is that the fixed-point operations (including the constructors) are implemented
    correctly. However, what this rigorously means is not obvious because it requires the mathematical model of
    fixed-point operations to check the implementations against. Mathematically, a finite-precision arithmetic operation
    (like the fixed-point one's) can be defined using the notion of "exact rounding". This means that such an operation
    is defined as the corresponding operation over real numbers followed by a pre-defined rounding operation which maps
    the resulting real number into a certain finite-precision number (depending on the rounding mode).

    From this viewpoint, the local specifications of this module can be defined as follows:
    * Let FP32 refer to FixedPoint32::T which is the 64-bit unsigned fixed-point numeric type with 32 factional bits.
    * Let FP0 refer to u64 which can be seen as the 64-bit unsigned fixed-point numeric type with 0 factional bits.
    * round_FP32: R -> FP32 is a function that takes a real number as an input and returns the largest FP32 number which
      is less than or equal to the input.
    * round_FP0: R -> FP0 is a function that takes a real number as an input and returns the largest FP0 number which is
      less than or equal to the input.
    * Let mul_R and div_R be the multiplication and the division function over real numbers respectively.
    */
    /*
    spec fun multiply_u64 {
        // aborts_if mul_R(num, multiplier) >= max_u64() + 1; // overflow (note that max_u64() == max_FP0())
        // aborts_if 0 < mul_R(num, multiplier) < 1; // underflow (note that 1 is the smallest non-zero FP0);
        // ensures result == round_FP0(mul_R(num, multiplier)); // exactly rounded
        pragma intrinsic = true;
    }

    spec fun divide_u64 {
        // aborts_if div_R(num, divisor) >= max_u64() + 1; // overflow
        // aborts_if 0 < div_R(num, divisor) < 1; // underflow
        // aborts_if divisor == 0; // div by zero
        // ensures result == round_FP0(div_R(num, divisor)); // exactly rounded
        pragma intrinsic = true;
    }

    spec fun create_from_rational {
        // aborts_if denominator == 0; // div by zero
        // aborts_if 0 < div_R(numerator, denominator) < smallest_non_zero_FP32(); // underflow
        // ensures result == round_FP32(div_R(numerator, denominator)); // exactly rounded
        pragma intrinsic = true;
    }

    spec fun create_from_raw_value {
        aborts_if false;
        ensures result == FixedPoint32 { value };
    }

    spec fun get_raw_value {
        aborts_if false;
        ensures result == num.value;
    }
    */

    /*
    Another approach to assuring this module is perhaps not formally verifying it, but carefully reviewing it and
    arguing its correctness informally (or looking for some pragmatic solution). It is because quite a bit of efforts is
    expected in rigorously specifying and formally verifying it, but it is not clear for now how much the benefit and
    the impact are compared to the expected effort. Currently, this module (FixedPoint32) is used in the following
    places (note that this is the complete list):

    coin1.move
    15:            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR

    coin2.move
    15:            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR

    libra.move
    73:        to_lbr_exchange_rate: FixedPoint32::FixedPoint32,
    446:        to_lbr_exchange_rate: FixedPoint32::FixedPoint32,
    489:        FixedPoint32::multiply_u64(from_value, lbr_exchange_rate)
    535:        lbr_exchange_rate: FixedPoint32::FixedPoint32
    543:    public fun lbr_exchange_rate<CoinType>(): FixedPoint32::FixedPoint32

    lbr.move
    19:        ratio: FixedPoint32::FixedPoint32,
    43:            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
    51:            ratio: FixedPoint32::create_from_rational(1, 2),
    55:            ratio: FixedPoint32::create_from_rational(1, 2),
    72:        let lbr_num_coin1 = FixedPoint32::divide_u64(coin1_value - 1, *&reserve.coin1.ratio);
    73:        let lbr_num_coin2 = FixedPoint32::divide_u64(coin2_value - 1, *&reserve.coin2.ratio);
    93:        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
    94:        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
    111:        let coin1_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin1.ratio);
    112:        let coin2_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin2.ratio);
    122:        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
    123:        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
    */
}

}
