module M {
    t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        while (cond) {
            _ = x_ref;
        };
        _ = x;
        _ = move x;
    }

    t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
           _ = x_ref;
           break
        };
        _ = x;
        _ = move x;
    }

    t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
            if (cond) break else  { _ = copy x_ref; }
        };
        _ = x;
        _ = move x;
    }

    t4(cond: bool) {
        let x = cond;
        let x_ref = &x;
        while (*x_ref) {

        };
        _ = x;
        _ = move x;
    }

}
