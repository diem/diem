module M {
    struct S { g: u64 }

    fun t1(root: &mut S, cond: bool) {
        let x1 = 0;
        let eps = if (cond) bar(root) else &x1;
        let g = &root.g;
        eps;
        g;
        eps;
    }

    fun t2() {
        let x1 = 0;
        let x2 = 1;
        let eps = foo(&x1, &x2);
        baz(&x1, eps);
    }

    fun t3() {
        let x1 = 0;
        let x2 = 1;
        let eps = foo(&x1, &x2);
        baz(freeze(&mut x1), eps);
    }

    fun foo(a: &u64, b: &u64): &u64 {
        let ret;
        if (*a > *b) {
            ret = move a;
            move b;
        } else {
            ret = move b;
            move a;
        };
        ret
    }

    fun bar(a: &mut S): &u64 {
        &a.g
    }

    fun baz(a: &u64, b: &u64) {
    }
}
