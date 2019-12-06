module M {
    tmove(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let y = move x + 1;
    }

    tcopy(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let y = move x + 1;
    }

    tborrow(cond: bool) {
        let x: u64;
        if (cond) { } else { x = 0 };
        let y = move x + 1;
    }

}
