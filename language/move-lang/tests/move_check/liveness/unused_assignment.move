module 0x8675309::M {
    fun t0() {
        let x = 0;
    }

    fun t1() {
        let x = 0;
        x = 0;
    }

    fun t2(cond: bool) {
        if (cond) {
            let x = 0;
        }
    }

    fun t3(cond: bool) {
        let x = 0;
        x;
        if (cond) {
            x = 0;
        }
    }

    fun t4(cond: bool) {
        let x = 0;
        if (cond) {
            x = 1;
        } else {
            x = 2;
        }
    }

    fun t5(cond: bool) {
        let x;
        while (cond) {
            x = 0;
            if (cond) {
                x;
            };
            x = 1;
        }
    }

}
