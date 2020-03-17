module TestCast {

    // --------------
    // Type promotion
    // --------------

    fun u8_cast_bad(x: u8): u64 {
        (x as u64)
    }
    spec fun u8_cast_bad {
        aborts_if false;
    }

    fun u64_cast_ok(x: u64): u128 {
        (x as u128)
    }
    spec fun aborting_u64_cast_ok {
        aborts_if false;
    }


    // -------------
    // Type demotion
    // -------------

    fun aborting_u8_cast_bad(x: u64): u8 {
        (x as u8)
    }
    spec fun aborting_u8_cast_bad {
        aborts_if false;
    }

    fun aborting_u8_cast_ok(x: u64): u8 {
        (x as u8)
    }
    spec fun aborting_u8_cast_ok {
        aborts_if x > 255;
    }

    fun aborting_u64_cast_bad(x: u128): u64 {
        (x as u64)
    }
    spec fun aborting_u64_cast_bad {
        aborts_if false;
    }

    fun aborting_u64_cast_ok(x: u128): u64 {
        (x as u64)
    }
    spec fun aborting_u64_cast_ok {
        aborts_if x > 18446744073709551615;
    }
}
