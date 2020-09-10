address 0x1 {

module FixedPoint32 {

    use 0x1::Errors;

    /// Define a fixed-point numeric type with 32 fractional bits.
    /// This is just a u64 integer but it is wrapped in a struct to
    /// make a unique type. This is a binary representation, so decimal
    /// values may not be exactly representable, but it provides more
    /// than 9 decimal digits of precision both before and after the
    /// decimal point (18 digits total). For comparison, double precision
    /// floating-point has less than 16 decimal digits of precision, so
    /// be careful about using floating-point to convert these values to
    /// decimal.
    struct FixedPoint32 { value: u64 }

    /// TODO(wrwg): This should be provided somewhere centrally in the framework.
    const MAX_U64: u128 = 18446744073709551615;

    /// The denominator provided was zero
    const EDENOMINATOR: u64 = 0;
    /// The quotient value would be too large to be held in a `u64`
    const EDIVISION: u64 = 1;
    /// The multiplied value would be too large to be held in a `u64`
    const EMULTIPLICATION: u64 = 2;
    /// A division by zero was encountered
    const EDIVISION_BY_ZERO: u64 = 3;
    /// The computed ratio when converting to a `FixedPoint32` would be unrepresentable
    const ERATIO_OUT_OF_RANGE: u64 = 4;

    /// Multiply a u64 integer by a fixed-point number, truncating any
    /// fractional part of the product. This will abort if the product
    /// overflows.
    public fun multiply_u64(val: u64, multiplier: FixedPoint32): u64 {
        // The product of two 64 bit values has 128 bits, so perform the
        // multiplication with u128 types and keep the full 128 bit product
        // to avoid losing accuracy.
        let unscaled_product = (val as u128) * (multiplier.value as u128);
        // The unscaled product has 32 fractional bits (from the multiplier)
        // so rescale it by shifting away the low bits.
        let product = unscaled_product >> 32;
        // Check whether the value is too large.
        assert(product <= MAX_U64, Errors::limit_exceeded(EMULTIPLICATION));
        (product as u64)
    }
    spec fun multiply_u64 {
        pragma opaque;
        include MultiplyAbortsIf;
        ensures result == spec_multiply_u64(val, multiplier);
    }
    spec schema MultiplyAbortsIf {
        val: num;
        multiplier: FixedPoint32;
        aborts_if spec_multiply_u64(val, multiplier) > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }
    spec define spec_multiply_u64(val: num, multiplier: FixedPoint32): num {
        (val * multiplier.value) >> 32
    }

    /// Divide a u64 integer by a fixed-point number, truncating any
    /// fractional part of the quotient. This will abort if the divisor
    /// is zero or if the quotient overflows.
    public fun divide_u64(val: u64, divisor: FixedPoint32): u64 {
        // Check for division by zero.
        assert(divisor.value != 0, Errors::invalid_argument(EDIVISION_BY_ZERO));
        // First convert to 128 bits and then shift left to
        // add 32 fractional zero bits to the dividend.
        let scaled_value = (val as u128) << 32;
        let quotient = scaled_value / (divisor.value as u128);
        // Check whether the value is too large.
        assert(quotient <= MAX_U64, Errors::limit_exceeded(EDIVISION));
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (quotient as u64)
    }
    spec fun divide_u64 {
        pragma opaque;
        include DividedAbortsIf;
        ensures result == spec_divide_u64(val, divisor);
    }
    spec schema DividedAbortsIf {
        val: num;
        divisor: FixedPoint32;
        aborts_if divisor.value == 0 with Errors::INVALID_ARGUMENT;
        aborts_if spec_divide_u64(val, divisor) > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }
    spec define spec_divide_u64(val: num, divisor: FixedPoint32): num {
        (val << 32) / divisor.value
    }

    /// Create a fixed-point value from a rational number specified by its
    /// numerator and denominator. This function is for convenience; it is also
    /// perfectly fine to create a fixed-point value by directly specifying the
    /// raw value. This will abort if the denominator is zero. It will also
    /// abort if the numerator is nonzero and the ratio is not in the range
    /// 2^-32 .. 2^32-1. When specifying decimal fractions, be careful about
    /// rounding errors: if you round to display N digits after the decimal
    /// point, you can use a denominator of 10^N to avoid numbers where the
    /// very small imprecision in the binary representation could change the
    /// rounding, e.g., 0.0125 will round down to 0.012 instead of up to 0.013.
    public fun create_from_rational(numerator: u64, denominator: u64): FixedPoint32 {
        // If the denominator is zero, this will abort.
        // Scale the numerator to have 64 fractional bits and the denominator
        // to have 32 fractional bits, so that the quotient will have 32
        // fractional bits.
        let scaled_numerator = (numerator as u128) << 64;
        let scaled_denominator = (denominator as u128) << 32;
        assert(scaled_denominator != 0, Errors::invalid_argument(EDENOMINATOR));
        let quotient = scaled_numerator / scaled_denominator;
        assert(quotient != 0 || numerator == 0, Errors::invalid_argument(ERATIO_OUT_OF_RANGE));
        // Return the quotient as a fixed-point number. We first need to check whether the cast
        // can succeed.
        assert(quotient <= MAX_U64, Errors::limit_exceeded(ERATIO_OUT_OF_RANGE));
        FixedPoint32 { value: (quotient as u64) }
    }
    spec fun create_from_rational {
        pragma opaque;
        include CreateFromRationalAbortsIf;
        ensures result == spec_create_from_rational(numerator, denominator);
    }
    spec schema CreateFromRationalAbortsIf {
        numerator: u64;
        denominator: u64;
        let scaled_numerator = numerator << 64;
        let scaled_denominator = denominator << 32;
        aborts_if scaled_denominator == 0 with Errors::INVALID_ARGUMENT;
        let quotient = scaled_numerator / scaled_denominator;
        aborts_if (scaled_numerator / scaled_denominator) == 0 && scaled_numerator != 0 with Errors::INVALID_ARGUMENT;
        aborts_if quotient > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }
    spec define spec_create_from_rational(numerator: num, denominator: num): FixedPoint32 {
        FixedPoint32{value: (numerator << 64) / (denominator << 32)}
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

    /// Returns true if the ratio is zero.
    public fun is_zero(num: FixedPoint32): bool {
        num.value == 0
    }

    // **************** SPECIFICATIONS ****************

    spec module {
        pragma verify;
        pragma aborts_if_is_strict = true;
    }

}

}
