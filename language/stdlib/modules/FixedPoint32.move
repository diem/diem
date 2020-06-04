address 0x0 {

module FixedPoint32 {
    use 0x0::Transaction;

    // Define a fixed-point numeric type with 32 fractional bits.
    // This is just a u64 integer but it is wrapped in a struct to
    // make a unique type.
    struct T { value: u64 }

    // Multiply a u64 integer by a fixed-point number, truncating any
    // fractional part of the product. This will abort if the product
    // overflows.
    public fun multiply_u64(num: u64, multiplier: T): u64 {
        // The product of two 64 bit values has 128 bits, so perform the
        // multiplication with u128 types and keep the full 128 bit product
        // to avoid losing accuracy.
        let unscaled_product = (num as u128) * (multiplier.value as u128);
        // The unscaled product has 32 fractional bits (from the multiplier)
        // so rescale it by shifting away the low bits.
        let product = unscaled_product >> 32;
        // Convert back to u64. If the multiplier is larger than 1.0,
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (product as u64)
    }

    // Divide a u64 integer by a fixed-point number, truncating any
    // fractional part of the quotient. This will abort if the divisor
    // is zero or if the quotient overflows.
    public fun divide_u64(num: u64, divisor: T): u64 {
        // First convert to 128 bits and then shift left to
        // add 32 fractional zero bits to the dividend.
        let scaled_value = (num as u128) << 32;
        // Divide and convert the quotient to 64 bits. If the divisor is zero,
        // this will fail with a divide-by-zero error.
        let quotient = scaled_value / (divisor.value as u128);
        // Convert back to u64. If the divisor is less than 1.0,
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (quotient as u64)
    }

    // Create a fixed-point value from a rational number specified by its
    // numerator and denominator. This function is for convenience; it is also
    // perfectly fine to create a fixed-point value by directly specifying the
    // raw value. This will abort if the denominator is zero or if the ratio is
    // not in the range 2^-32 .. 2^32-1.
    public fun create_from_rational(numerator: u64, denominator: u64): T {
        // Scale the numerator to have 64 fractional bits and the denominator
        // to have 32 fractional bits, so that the quotient will have 32
        // fractional bits.
        let scaled_numerator = (numerator as u128) << 64;
        let scaled_denominator = (denominator as u128) << 32;
        // If the denominator is zero, this will fail with a divide-by-zero
        // error.
        let quotient = scaled_numerator / scaled_denominator;
        // Check for underflow. Truncating to zero might be the desired result,
        // but if you really want a ratio of zero, it is easy to create that
        // from a raw value.
        Transaction::assert(quotient != 0 || numerator == 0, 16);
        // Return the quotient as a fixed-point number. The cast will fail
        // with an arithmetic error if the number is too large.
        T { value: (quotient as u64) }
    }

    public fun create_from_raw_value(value: u64): T {
        T { value }
    }

    // Accessor for the raw u64 value. Other less common operations, such as
    // adding or subtracting FixedPoint32 values, can be done using the raw
    // values directly.
    public fun get_raw_value(num: T): u64 {
        num.value
    }
}

}
