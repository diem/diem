module M {
    t0() {
        let x = 0;
        let x_ref = &mut x;
        _ = x;
        _ = move x;
        *x_ref = 0;
    }

    t1() {
        let x = 0;
        let x_ref = &mut x;
        _ = x;
        _ = move x;
        _ = *x_ref;
    }

}
