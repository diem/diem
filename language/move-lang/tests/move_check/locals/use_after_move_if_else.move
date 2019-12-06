module M {
    tmove1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let y = move x + 1;
    }

    tmove2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let y = move x + 1;
    }

    tcopy1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let y = x + 1;
    }

    tcopy2(cond: bool) {

        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let y = x + 1;
    }

    tborrow1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let y = &x;
    }

    tborrow2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let y = &x;
    }

}
