module TestArithmetic {

    // Most of the tests here just ensure that what the bytecode operation does is the same as the
    // spec expressions.

	fun add_two_number(x: u64, y: u64): (u64, u64) {
		let res: u64 = x + y;
		let z: u64 = 3;
		(z, res)
	}
	spec fun add_two_number {
	    aborts_if x + y > 9223372036854775807;
	    ensures result_1 == 3;
	    ensures result_2 == x + y;
	}

	fun multiple_ops(x: u64, y: u64, z: u64): u64 {
		x + y * z
	}
	spec fun multiple_ops {
        ensures result == x + y * z;
    }

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

	fun overflow() : u64 {
        let x = 9223372036854775807;
        x + 1
	}
	spec fun overflow {
	    aborts_if false;
	}

    fun underflow(): u64 {
        let x = 0;
        x - 1
    }
	spec fun underflow {
	    aborts_if false;
	}

    fun div_by_zero(): u64 {
        let x = 0;
        1 / x
    }
	spec fun div_by_zero {
	    aborts_if false;
	}
}
