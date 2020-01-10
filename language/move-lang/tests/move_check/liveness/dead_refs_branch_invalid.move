module M {
    t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
        *x_ref = 0;
    }

    t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
        } else {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
        _ = *x_ref;
    }

}
