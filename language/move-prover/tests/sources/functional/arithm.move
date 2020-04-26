module TestArithmetic {

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

    fun div(x: u64, y: u64): (u64, u64) {
        (x / y, x % y)
    }
    spec fun div {
        aborts_if y == 0;
        ensures result_1 == x / y;
        ensures result_2 == x % y;
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
	fun arithmetic_ops(a: u64): (u64, u64) {
        let c: u64;
        c = (6 + 4 - 1) * 2 / 3 % 4;
        if (c != 2) abort 42;
        (c, a)
    }
    spec fun arithmetic_ops {
        ensures result_1 == (6 + 4 - 1) * 2 / 3 % 4;
        ensures result_2 == a;
    }

    fun f(x: u64) : u64 {
        x+1
    }
    spec fun f {
        aborts_if x+1 > max_u64();
        ensures result == x+1;
    }

    fun g(x: u64) : u64 {
        x+2
    }
    spec fun g {
        aborts_if x+2 > max_u64();
        ensures result == x+2;
    }

    fun h(b: bool): u64 {
        let x: u64 = 3;
        let y: u64;
        if (b) y=f(x) else y=g(x);
        if (b && y != 4) abort 4;
        if (!b && y != 5) abort 5;
        y
    }
    spec fun h {
        aborts_if false;
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
    fun div_by_zero(): u64 {
        let x = 0;
        1 / x
    }
	spec fun div_by_zero {
	    aborts_if true;
	}

    fun div_by_zero_u64_incorrect(x: u64, y: u64): u64 {
        x / y
    }
    spec fun div_by_zero_u64_incorrect {
        aborts_if false;
    }

    fun div_by_zero_u64(x: u64, y: u64): u64 {
        x / y
    }
    spec fun div_by_zero_u64 {
        aborts_if y == 0;
    }


    // --------------------
    // Overflow by addition
    // --------------------

    // fails.
    fun overflow_u8_add_incorrect(x: u8, y: u8): u8 {
        x + y
    }
    spec fun overflow_u8_add_incorrect {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u8_add(x: u8, y: u8): u8 {
        x + y
    }
    spec fun overflow_u8_add {
        aborts_if x + y > max_u8();
    }

    // fails.
    fun overflow_u64_add_incorrect(x: u64, y: u64): u64 {
        x + y
    }
    spec fun overflow_u64_add_incorrect {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u64_add(x: u64, y: u64): u64 {
        x + y
    }
    spec fun overflow_u64_add {
        aborts_if x + y > max_u64();
    }

    // fails.
    fun overflow_u128_add_incorrect(x: u128, y: u128): u128 {
        x + y
    }
    spec fun overflow_u128_add_incorrect {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u128_add(x: u128, y: u128): u128 {
        x + y
    }
    spec fun overflow_u128_add {
        aborts_if x + y > max_u128();
    }


    // --------------------------
    // Overflow by multiplication
    // --------------------------

    // fails.
    fun overflow_u8_mul_incorrect(x: u8, y: u8): u8 {
        x * y
    }
    spec fun overflow_u8_mul_incorrect {
        aborts_if false;
    }

    // succeeds.
    fun overflow_u8_mul(x: u8, y: u8): u8 {
        x * y
    }
    spec fun overflow_u8_mul {
        aborts_if x * y > max_u8();
    }

    // fails.
    fun overflow_u64_mul_incorrect(x: u64, y: u64): u64 {
        x * y
    }
    spec fun overflow_u64_mul_incorrect {
        aborts_if false;
    }

    // TODO: Should succeed. JP: Prover seems to give a false counterexample.
    fun overflow_u64_mul(x: u64, y: u64): u64 {
        x * y
    }
    spec fun overflow_u64_mul {
        //aborts_if x * y > max_u64();
    }

    // fails.
    fun overflow_u128_mul_incorrect(x: u128, y: u128): u128 {
        x * y
    }
    spec fun overflow_u128_mul_incorrect {
        aborts_if false;
    }

    // TODO: Should succeed. JP: Prover seems to give a false counterexample.
    fun overflow_u128_mul(x: u128, y: u128): u128 {
        x * y
    }
    spec fun overflow_u128_mul {
        //aborts_if x * y > max_u128(); // U128_MAX
    }
}
