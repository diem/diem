module 0x8675309::M {
    fun tmove(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = move x + 1;
    }

    fun tcopy(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = x + 1;
    }

    fun tborrow(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = &x;
    }

}
