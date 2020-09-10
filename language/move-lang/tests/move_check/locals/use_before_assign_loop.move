module M {
    fun tmove() {
        let x: u64;
        loop { let y = move x + 1; x = 0; y; }
    }

    fun tcopy(cond: bool) {
        let x: u64;
        loop { let y = x + 1; if (cond) { continue }; x = 0; y; }
    }

    fun tborrow1() {
        let x: u64;
        loop { let y = &x; _ = move y; x = 0 }
    }

    fun tborrow2(cond: bool) {
        let x: u64;
        loop { let y = &x; _ = move y; if (cond) { x = 0 }; break };
        x;
    }

}
