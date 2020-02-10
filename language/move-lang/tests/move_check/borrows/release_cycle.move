module M {
    fun t0(cond: bool) {
        let v = 0;
        let (x, y);
        if (cond) {
            x = &v;
            y = copy x;
        } else {
            y = &v;
            x = copy y;
        };
        move x;
        move y;
    }
}
