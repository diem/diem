module M {
    tmove(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = move x + 1;
    }

    tcopy(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = x + 1;
    }

    tborrow(cond: bool) {
        let x = 0;
        if (cond) { _ = move x };
        let _ = &x;
    }

}
