module M {
    t0(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        while (cond) {
            while (cond) {
                _ = x_ref
            }
        };
        _ = x;
        _ = move x;
    }

    t1(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
           _ = x_ref;
           loop {
               _ = x_ref;
               break
           };
           break
        };
        _ = x;
        _ = move x;
    }

    t2(cond: bool) {
        let x = 0;
        let x_ref = &mut x;
        loop {
            if (cond) { _ = x; break } else { while (!cond) { _ = x_ref } }
        }
    }
}
