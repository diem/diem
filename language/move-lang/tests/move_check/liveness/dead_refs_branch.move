module M {
    t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
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
    }

    t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *move x_ref = 0;
        };
        _ = x;
        _ = move x;
    }

    t3(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *move x_ref = 0;
        } else {
            _ = move x_ref;
        };
        _ = x;
        _ = move x;
    }

}
