module TestEliminateImmRefs {

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

    fun test2() : u64 {
        let r = R {x: 3};
        let r_ref = & r;
        let x_ref = & r_ref.x;
        *x_ref
    }

    fun test3(r_ref: &R) : u64 {
        let x_ref = & r_ref.x;
        *x_ref
    }

    fun test4() : u64 {
        let r = R {x: 3};
        let r_ref = & r;
        test3(r_ref)
    }
}
