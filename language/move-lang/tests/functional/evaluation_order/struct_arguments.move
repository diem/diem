//! new-transaction

module M {
    struct S {
        a: u64,
        b: u64,
    }

    resource struct R {}
    resource struct Cup {
        a: u64,
        b: R,
    }

    public t0() {
        S { b: 1 / 0, a: fail(0) };
    }

    public t1() {
        S { b: 18446744073709551615 + 18446744073709551615, a: fail(0) };
    }

    public t2() {
        S { b: 0 - 1, a: fail(0) };
    }

    public t3() {
        S { b: 1 % 0, a: fail(0) };
    }

    public t4() {
        S { b: 18446744073709551615 * 18446744073709551615, a: fail(0) };
    }

    public t5() acquires R {
        move_to_sender(Cup { b: move_from(0x0), a: fail(0) });
    }

    public t6() {
        move_to_sender(Cup { b: R{}, a: 0 });
        S { b: mts(), a: fail(0) };
    }

    fail(code: u64): u64 {
        abort code
    }

    mts(): u64 {
        move_to_sender(Cup { b: R{}, a: 0 });
        0
    }
}

//! new-transaction
// check: ARITHMETIC_ERROR
use {{default}}::M;
main() {
  M::t0()
}

//! new-transaction
// check: ARITHMETIC_ERROR
use {{default}}::M;
main() {
  M::t1()
}

//! new-transaction
// check: ARITHMETIC_ERROR
use {{default}}::M;
main() {
  M::t2()
}

//! new-transaction
// check: ARITHMETIC_ERROR
use {{default}}::M;
main() {
  M::t3()
}

//! new-transaction
// check: ARITHMETIC_ERROR
use {{default}}::M;
main() {
  M::t4()
}

//! new-transaction
// check: MISSING_DATA
use {{default}}::M;
main() {
  M::t5()
}

//! new-transaction
// check: CANNOT_WRITE_EXISTING_RESOURCE
use {{default}}::M;
main() {
  M::t6()
}
