module M {
    struct X { f: Y }
    struct Y { g: u64, h: u64 }

    fun t1() {
        let x = X { f: Y { g: 0, h: 0 } };
        let g = &mut x.f.g;
        let h = &mut x.f.h;

        *g = *h;
        *h = *g;

        foo(g, h);
    }

    fun foo(a: &mut u64, b: &mut u64) {
    }
}
