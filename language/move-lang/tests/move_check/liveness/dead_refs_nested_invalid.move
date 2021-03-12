module 0x8675309::M {
    fun t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        while (cond) {
            while (cond) {
                _ = x_ref
            };
            _ = x;
        }
    }

    fun t1() {
        let x = 0;
        let x_ref = &mut x;
        loop {
           _ = x_ref;
           loop {
               _ = x;
               break
           }
        }
    }

    fun t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
            if (cond) { _ = x_ref; break } else { while (!cond) { _ = x } }
        }
    }
}
