module M {
    tmove1(cond: bool) {
        let x = 0;
        while (cond) { _ = move x };
    }

    tmove2(cond: bool) {
        let x = 0;
        while (cond) { if (cond) break; _ = move x };
    }

    tcopy1(cond: bool) {
        let x = 0;
        while (cond) { let y = x; _ = move x };
    }

    tcopy2(cond: bool) {
        let x = 0;
        while (cond) { let y = x; if (cond) continue; _ = move x };
    }

    tborrow1(cond: bool) {
        let x = 0;
        while (cond) { let y = &x; _ = move y; _ = move x };
    }

    tborrow2(cond: bool) {
        let x = 0;
        while (cond) { let y = &x; _ = move y; if (cond) {_ = move x }; break };
    }

}
