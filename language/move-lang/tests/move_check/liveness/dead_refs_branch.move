module M {
    fun t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
    }

    fun t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
        } else {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
    }

    fun t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *move x_ref = 0;
        };
        _ = x;
        _ = move x;
    }

    fun t3(cond: bool) {
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

    fun t4(cond: bool) {
        let x = cond;
        let x_ref = &x;
        if (*x_ref) {
        } else {
        };
        _ = x;
        _ = move x;
    }
}
