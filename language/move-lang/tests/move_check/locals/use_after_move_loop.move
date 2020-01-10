module M {
    tmove1(cond: bool) {
        let x = 0;
        loop { _ = move x };
    }

    tmove2(cond: bool) {
        let x = 0;
        loop { if (cond) break; _ = move x }
    }

    tcopy1(cond: bool) {
        let x = 0;
        loop { let y = x; _ = move x; y; }
    }

    tcopy2(cond: bool) {
        let x = 0;
        loop { let y = x; if (cond) continue; _ = move x; y; }
    }

    tborrow1(cond: bool) {
        let x = 0;
        loop { let y = &x; _ = move y; _ = move x }
    }

    tborrow2(cond: bool) {
        let x = 0;
        loop { let y = &x; _ = move y; if (cond) {_ = move x }; break }
    }

}
