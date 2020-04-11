module M {
    fun take_imm_mut_give_mut(_x: &u64, y: &mut u64): &mut u64 {
        y
    }

    fun take_imm_mut_give_imm(_x: &u64, y: &mut u64): &u64 {
        y
    }

    fun t0() {
        let x = 0;
        let y = 0;
        let x_ref = &x;
        let y_ref = &mut y;
        let r = take_imm_mut_give_mut(move x_ref, move y_ref);
        move y;
        *r = 1;
    }

    fun t1() {
        let x = 0;
        let y = 0;
        let x_ref = &x;
        let y_ref = &mut y;
        let r = take_imm_mut_give_imm(move x_ref, move y_ref);
        move x;
        move y;
        *r;
    }
}
