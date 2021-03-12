module 0x8675309::M {
    struct X has copy, drop { f: Y }
    struct Y has copy, drop { g: u64, h: u64 }

    fun t1() {
        let x = X { f: Y { g: 0, h: 0 } };
        let g = &mut x.f.g;
        let h = &mut x.f.h;

        *g = *h;
        *h = *g;

        foo(g, h);
    }

    fun foo(_a: &mut u64, _b: &mut u64) {
    }
}
