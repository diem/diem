module M {
    take_imm_mut_give_mut(x: &u64, y: &mut u64): &mut u64 {
        y
    }

    take_imm_mut_give_imm(x: &u64, y: &mut u64): &u64 {
        y
    }

    t0() {
        let x = 0;
        let y = 0;
        let x_ref = &x;
        let y_ref = &mut y;
        let r = take_imm_mut_give_mut(x_ref, y_ref);
        *x_ref;
        move x;
        *r = 1;
    }

    t1() {
        let x = 0;
        let y = 0;
        let x_ref = &x;
        let y_ref = &mut y;
        let r = take_imm_mut_give_imm(x_ref, y_ref);
        *r;
        *x_ref;
        *y_ref;
        *x_ref;
        *r;
    }

}
