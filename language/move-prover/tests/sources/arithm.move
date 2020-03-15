module Arithmetic {

    // --------------------------
    // Basic arithmetic operation
    // --------------------------

    // Most of the tests here just ensure that what the bytecode operation does is the same as the
    // spec expressions.

    // succeeds.
	fun add_two_number(x: u64, y: u64): (u64, u64) {
		let res: u64 = x + y;
		let z: u64 = 3;
		(z, res)
	}
	spec fun add_two_number {
	    aborts_if x + y > max_u64();
	    ensures result_1 == 3;
	    ensures result_2 == x + y;
	}

    // succeeds.
	fun multiple_ops(x: u64, y: u64, z: u64): u64 {
		x + y * z
	}
	spec fun multiple_ops {
        ensures result == x + y * z;
    }

    // succeeds.
	fun bool_ops(a: u64, b: u64): (bool, bool) {
        let c: bool;
        let d: bool;
        c = a > b && a >= b;
        d = a < b || a <= b;
        if (!(c != d)) abort 42;
        (c, d)
    }
    spec fun bool_ops {
        ensures result_1 == (a > b && a >= b);
        ensures result_2 == (a < b || a <= b);
    }

    // succeeds.
	fun arithmetic_ops(a: u64, b: u64): (u64, u64) {
        let c: u64;
        c = (6 + 4 - 1) * 2 / 3 % 4;
        if (c != 2) abort 42;
        (c, a)
    }
    spec fun arithmetic_ops {
        ensures result_1 == (6 + 4 - 1) * 2 / 3 % 4;
        ensures result_2 == a;
    }


    // ---------
    // Underflow
    // ---------

    // succeeds.
    fun underflow(): u64 {
        let x = 0;
        x - 1
    }
	spec fun underflow {
	    aborts_if true;
	}


    // ----------------
    // Division by zero
    // ----------------

    // succeeds.
    fun div_by_zero_ok(): u64 {
        let x = 0;
        1 / x
    }
	spec fun div_by_zero_ok {
	    aborts_if true;
	}

    fun div_by_zero_u64_bad(x: u64, y: u64): u64 {
        x / y
    }
    spec fun div_by_zero_u64_bad {
        aborts_if false;
    }

    fun div_by_zero_u64_ok(x: u64, y: u64): u64 {
        x / y
    }
    spec fun div_by_zero_u64_ok {
        aborts_if y == 0;
    }


    // --------------------
    // Overflow by addition
    // --------------------

    // fails.
    fun overflow_u8_add_bad(x: u8, y: u8): u8 {
        x + y
    }
    spec fun overflow_u8_add_bad {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u8_add_ok(x: u8, y: u8): u8 {
        x + y
    }
    spec fun overflow_u8_add_ok {
        aborts_if x + y > max_u8();
    }

    // fails.
    fun overflow_u64_add_bad(x: u64, y: u64): u64 {
        x + y
    }
    spec fun overflow_u64_add_bad {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u64_add_ok(x: u64, y: u64): u64 {
        x + y
    }
    spec fun overflow_u64_add_ok {
        aborts_if x + y > max_u64();
    }

    // fails.
    fun overflow_u128_add_bad(x: u128, y: u128): u128 {
        x + y
    }
    spec fun overflow_u128_add_bad {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u128_add_ok(x: u128, y: u128): u128 {
        x + y
    }
    spec fun overflow_u128_add_ok {
        aborts_if x + y > max_u128();
    }


    // --------------------------
    // Overflow by multiplication
    // --------------------------

    // fails.
    fun overflow_u8_mul_bad(x: u8, y: u8): u8 {
        x * y
    }
    spec fun overflow_u8_mul_bad {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u8_mul_ok(x: u8, y: u8): u8 {
        x * y
    }
    spec fun overflow_u8_mul_ok {
        aborts_if x * y > max_u8();
    }

    // fails.
    fun overflow_u64_mul_bad(x: u64, y: u64): u64 {
        x * y
    }
    spec fun overflow_u64_mul_bad {
        aborts_if false;
    }

    // TODO: Should succeed. JP: Prover seems to give a false counterexample.
    fun overflow_u64_mul_ok(x: u64, y: u64): u64 {
        x * y
    }
    spec fun overflow_u64_mul_ok {
        //aborts_if x * y > max_u64();
    }

    // fails.
    fun overflow_u128_mul_bad(x: u128, y: u128): u128 {
        x * y
    }
    spec fun overflow_u128_mul_bad {
        aborts_if false;
    }

    // TODO: Should succeed. JP: Prover seems to give a false counterexample.
    fun overflow_u128_mul_ok(x: u128, y: u128): u128 {
        x * y
    }
    spec fun overflow_u128_mul_ok {
        //aborts_if x * y > max_u128(); // U128_MAX
    }
}
