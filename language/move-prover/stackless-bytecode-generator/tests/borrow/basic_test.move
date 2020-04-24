module TestBorrow {
    struct R {
        x: u64
    }

    fun test1() : R {
        let r = R {x: 3};
        let r_ref = &mut r;
        let x_ref = &mut r_ref.x;
        *x_ref = 0;
        r
    }

    fun test2(x_ref: &mut u64, v: u64) {
        *x_ref = v
    }

    public fun test3(r_ref: &mut R, v: u64) {
        let x_ref = &mut r_ref.x;
        test2(x_ref, v)
    }

    fun test4() : R {
        let r = R {x: 3};
        let r_ref = &mut r;
        test3(r_ref, 0);
        r
    }

    public fun test5(r_ref: &mut R) : &mut u64 {
        &mut r_ref.x
    }

    fun test6() : R {
        let r = R {x: 3};
        let r_ref = &mut r;
        let x_ref = test5(r_ref);
        test2(x_ref, 0);
        r
    }

    fun test7(b: bool) {
        let r1 = R {x: 3};
        let r2 = R {x: 4};
        let r_ref = &mut r1;
        if (b) {
            r_ref = &mut r2;
        };
        test3(r_ref, 0)
    }

    fun test8(n: u64, r_ref: &mut R) {
        let r1 = R {x: 3};
        let r2 = R {x: 4};
        while (0 < n) {
            if (n/2 == 0) {
                r_ref = &mut r1
            } else {
                r_ref = &mut r2;
            };
            n = n - 1
        };
        test3(r_ref, 0)
    }
}
