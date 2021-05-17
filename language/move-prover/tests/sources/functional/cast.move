module 0x42::TestCast {

    spec module {
        pragma verify = true;
    }

    // --------------
    // Type promotion
    // --------------

    fun u8_cast_incorrect(x: u8): u64 {
        (x as u64)
    }
    spec u8_cast_incorrect {
        aborts_if false;
    }

    fun u64_cast(x: u64): u128 {
        (x as u128)
    }
    spec aborting_u64_cast {
        aborts_if false;
    }


    // -------------
    // Type demotion
    // -------------

    fun aborting_u8_cast_incorrect(x: u64): u8 {
        (x as u8)
    }
    spec aborting_u8_cast_incorrect {
        aborts_if false;
    }

    fun aborting_u8_cast(x: u64): u8 {
        (x as u8)
    }
    spec aborting_u8_cast {
        aborts_if x > 255;
    }

    fun aborting_u64_cast_incorrect(x: u128): u64 {
        (x as u64)
    }
    spec aborting_u64_cast_incorrect {
        aborts_if false;
    }

    fun aborting_u64_cast(x: u128): u64 {
        (x as u64)
    }
    spec aborting_u64_cast {
        aborts_if x > 18446744073709551615;
    }
}
