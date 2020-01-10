module M {
    t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        while (cond) {
            _ = x;
            _ = x_ref;
        }
    }

    t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
           _ = x_ref;
           _ = x;
        }
    }

    t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
            if (cond) break else  { _ = x_ref; };
            _ = x;
        }
    }
}
