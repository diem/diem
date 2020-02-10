module M {
    fun t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *x_ref = 0;
        } else {
            _ = x_ref;
        };
        _ = x;
        _ = move x;
        *x_ref = 0;
    }

    fun t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            _ = x_ref;
        } else {
            *x_ref = 0;
        };
        _ = x;
        _ = move x;
        _ = *x_ref;
    }

}
