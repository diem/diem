module 0x8675309::M {
    fun t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
            *move x_ref = 1;
        };
        x = 1;
        x;
    }

    fun t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        if (cond) {
        } else {
            *move x_ref = 1;
        };
        x = 1;
        x;
    }
}
