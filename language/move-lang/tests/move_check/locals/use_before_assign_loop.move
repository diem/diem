module M {
    tmove(cond: bool) {
        let x: u64;
        loop { let y = move x + 1; x = 0; y; }
    }

    tcopy(cond: bool) {
        let x: u64;
        loop { let y = x + 1; if (cond) { continue }; x = 0; y; }
    }

    tborrow1(cond: bool) {
        let x: u64;
        loop { let y = &x; _ = move y; x = 0 }
    }

    tborrow2(cond: bool) {
        let x: u64;
        loop { let y = &x; _ = move y; if (cond) { x = 0 }; break };
        x;
    }

}
