module M {
    tmove1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = move x + 1;
    }

    tmove2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = move x + 1;
    }

    tcopy1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = x + 1;
    }

    tcopy2(cond: bool) {

        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = x + 1;
    }

    tborrow1(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = move x };
        let _ = &x;
    }

    tborrow2(cond: bool) {
        let x = 0;
        if (cond) { _ = move x } else { _ = x };
        let _ = &x;
    }

}
