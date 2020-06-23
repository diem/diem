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
    // Everything is specified so the boogie file should return errors
    // for all checks and the move prover should report nothing.
    public fun mod_T_x(t: &mut T)
    {
        let x = &mut t.x;
        *x = 10;
    }
    spec fun mod_T_x {
        ensures t.x == 10;
        ensures t.y == old(t.y);
    }

    // Tests whether missing specificaitons are reported
    // There should be three reports. One for t.y and
    // two for the fields of _t2.
    public fun mod_T_x_incorrect(t: &mut T, _t2: &mut T)
    {
        let x = &mut t.x;
        *x = 10;
    }
    spec fun mod_T_x_incorrect {
        ensures t.x == 10;
        // ensures t.y == old(t.y);
    }

    public fun multi_returns(cond: bool, t: &mut T): (u64, u64) {
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        if (cond) {
            let x = &mut t.x;
            *x = 10;
            return (a, b)
        } else {
            (c, d)
        }
    }
    spec fun multi_returns {
        ensures cond ==> t.x == 10 && t.y == old(t.y);
        ensures cond ==> result_1 == 1 && result_2 == 2;
        // If the line below is removed, then no warnings occur
        // because result_1 is specified above, but there should
        // be a warning. The check should find underspecified
        // postconditions for all control paths.
        // TODO: (kkmc) > Generate a check at every control path.
        ensures !cond ==> result_2 == 3 && result_2 == 4;
    }

    // Tests whether function calls break anything.
    public fun mod_T_x_callee(t: &mut T): u64 {
        let x = &mut t.x;
        *x = 0;
        0
    }
    spec fun mod_T_x_callee {
        ensures t.x == 0;
        ensures t.y == old(t.y);
        ensures result == 0;
    }

    /// Should fail because t isn't specified
    public fun mod_T_x_caller(t: &mut T): u64 {
        let y = mod_T_x_callee(t);
        y
    }
    spec fun mod_T_x_caller {
        // ensures t.x == 0;
        // ensures t.y == old(t.y);
    }
}
