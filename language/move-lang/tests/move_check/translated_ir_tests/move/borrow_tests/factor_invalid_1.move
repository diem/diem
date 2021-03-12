module 0x8675309::M {
    struct X has copy, drop { f: Y }
    struct Y has copy, drop { g: u64, h: u64 }

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

    fun foo(_a: &mut u64, _b: &mut Y) {
    }

    fun bar(_a: &mut Y, _b: &mut u64) {
    }
}
