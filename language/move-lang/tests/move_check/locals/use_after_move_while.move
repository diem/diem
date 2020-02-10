module M {
    fun tmove1(cond: bool) {
        let x = 0;
        while (cond) { _ = move x };
    }

    fun tmove2(cond: bool) {
        let x = 0;
        while (cond) { if (cond) break; _ = move x };
    }

    fun tcopy1(cond: bool) {
        let x = 0;
        while (cond) { let y = x; _ = move x; y; };
    }

    fun tcopy2(cond: bool) {
        let x = 0;
        while (cond) { let y = x; if (cond) continue; _ = move x; y; };
    }

    fun tborrow1(cond: bool) {
        let x = 0;
        while (cond) { let y = &x; _ = move y; _ = move x };
    }

    fun tborrow2(cond: bool) {
        let x = 0;
        while (cond) { let y = &x; _ = move y; if (cond) { _ = move x }; break };
    }

}
