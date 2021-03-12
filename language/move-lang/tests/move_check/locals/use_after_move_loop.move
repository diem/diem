module 0x8675309::M {
    fun tmove1() {
        let x = 0;
        loop { _ = move x }
    }

    fun tmove2(cond: bool) {
        let x = 0;
        loop { if (cond) break; _ = move x }
    }

    fun tcopy1() {
        let x = 0;
        loop { let y = x; _ = move x; y; }
    }

    fun tcopy2(cond: bool) {
        let x = 0;
        loop { let y = x; if (cond) continue; _ = move x; y; }
    }

    fun tborrow1() {
        let x = 0;
        loop { let y = &x; _ = move y; _ = move x }
    }

    fun tborrow2(cond: bool) {
        let x = 0;
        loop { let y = &x; _ = move y; if (cond) {_ = move x }; break }
    }

}
