module M {
    struct X { f: Y }
    struct Y { g: u64, h: u64 }

    fun t1() {
        let x = X { f: Y { g: 0, h: 0 } };

        let f = &mut x.f;
        let f_g = &mut f.g;

        // Error: the argument for parameter b is borrowed
        foo(f_g, f);
    }

    fun t2() {
        let x = X { f: Y { g: 0, h: 0 } };

        let f = &mut x.f;
        let f_g = &mut f.g;

        // Error: the argument for parameter a is borrowed
        bar(f, f_g);
    }

    fun foo(a: &mut u64, b: &mut Y) {
    }

    fun bar(a: &mut Y, b: &mut u64) {
    }
}

// check: CALL_BORROWED_MUTABLE_REFERENCE_ERROR
// check: CALL_BORROWED_MUTABLE_REFERENCE_ERROR
