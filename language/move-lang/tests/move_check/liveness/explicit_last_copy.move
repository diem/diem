module M {
    t0() {
        let x = 0;
        copy x;
    }

    t1() {
        let x = 0;
        let x = copy x;
        x;
    }

    t2(cond: bool) {
        if (cond) {
            let x = 0;
            copy x;
        }
    }

    t3(cond: bool) {
        let x = 0;
        copy x;
        if (cond) {
            copy x;
        }
    }

    t4(cond: bool) {
        let x = 0;
        if (cond) {
            copy x;
        } else {
            copy x;
        }
    }

    t5(cond: bool) {
        let x = 0;
        while (cond) {
            copy x;
        };
    }

    t6(cond: bool) {
        let x = 0;
        while (cond) {
            copy x;
        };
        copy x;
    }

}
