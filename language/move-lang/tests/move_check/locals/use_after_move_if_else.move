module 0x8675309::M {
    fun tmove1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = move x + 1;
    }

    fun tmove2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = move x + 1;
    }

    fun tcopy1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = x + 1;
    }

    fun tcopy2(cond: bool) {

        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = x + 1;
    }

    fun tborrow1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = &x;
    }

    fun tborrow2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = &x;
    }

}
