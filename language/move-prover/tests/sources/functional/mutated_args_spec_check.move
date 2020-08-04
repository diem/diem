module TestMutatedArgSpecCheck {
    spec module {
        //pragma verify = true;
        pragma verify = false;
        pragma mutated_args_test = true;
        // pragma writeref_test = true;
    }

    resource struct T {
        x: u64,
        y: u64,
    }

    // Tests whether mutable arguments are checked
    public fun mod_T_x(t: &mut T, _t2: &mut T, _v: &mut vector<u64>): u64
    {
        let x = &mut t.x;
        *x = 10;
        let x2 = &mut _t2.x;
        *x2 = 20;
        30
    }
    spec fun mod_T_x {
        ensures t.x == 10;
        ensures t.y == old(t.y);
        ensures _t2.x == 20;
        // ensures _t2.y == old(_t2.y);
        ensures result == 30;
    }

    // public fun multi_returns(cond: bool, t: &mut T): (u64, u64) {
    //     let a = 1;
    //     let b = 2;
    //     let c = 3;
    //     let d = 4;
    //     if (cond) {
    //         let x = &mut t.x;
    //         *x = 10;
    //         return (a, b)
    //     } else {
    //         (c, d)
    //     }
    // }

    // Tests whether functions are checked
    // public fun mod_T_x_callee(t: &mut T): u64 {
    //     let x = &mut t.x;
    //     *x = 0;
    //     0
    // }
    // spec fun mod_T_x_callee {
    //     ensures t.x == 0;
    //     ensures t.y == old(t.y);
    // }
    // public fun mod_T_x_caller(t: &mut T): u64 {
    //     let y = mod_T_x_callee(t);
    //     y
    // }
    // spec fun mod_T_x_caller {
    //     // ensures t.x == 0;
    //     // ensures t.y == old(t.y);
    // }

    // public fun mod_global_T_x(addr: address)
    // acquires T
    // {
    //     let t = borrow_global_mut<T>(addr);
    //     let x = &mut t.x;
    //     *x = 10;
    // }

    // public fun lots_of_branches(b: bool) {
    //     if (b) {
    //         if (b) {
    //             return
    //         } else {
    //             return
    //         }
    //     } else {
    //         return
    //     }
    // }



}
