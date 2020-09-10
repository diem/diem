module M {
    fun tmove(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let _ = move x + 1;
    }

    fun tcopy(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let _ = move x + 1;
    }

    fun tborrow(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let _ = move x + 1;
    }

}
