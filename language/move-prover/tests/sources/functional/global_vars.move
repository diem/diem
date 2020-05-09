module TestGlobalVars {
    spec module {
        pragma verify = true;
    }

    spec module {
        global sum_of_T: u64;
    }

    struct T {
      i: u64,
    }
    spec struct T {
      invariant pack sum_of_T = sum_of_T + i;
      invariant unpack sum_of_T = sum_of_T - i;
      invariant update sum_of_T = sum_of_T - old(i) + i;
    }


    // ----------
    // Pack tests
    // ----------

    fun pack_correct(): T {
        T {i: 2}
    }
    spec fun pack_correct {
        ensures sum_of_T == old(sum_of_T) + 2;
    }

    fun pack_incorrect(): T {
        T {i: 2}
    }
    spec fun pack_incorrect {
        ensures sum_of_T == old(sum_of_T) + 1;
        ensures result.i == 2;
    }


    // ------------
    // Unpack tests
    // ------------

    fun unpack_correct(t: T): u64 {
        let T {i: x} = t;
        x
    }
    spec fun unpack_correct {
        ensures sum_of_T == old(sum_of_T) - old(t.i);
        ensures result == old(t.i);
    }

    fun unpack_incorrect(t: T): u64 {
        let T {i: x} = t;
        x
    }
    spec fun unpack_incorrect {
        ensures sum_of_T == old(sum_of_T);
        ensures result == old(t.i);
    }


    // ------------
    // Update tests
    // ------------

    fun update_valid_still_mutating(t: &mut T) {
        t.i = t.i + 3;
    }
    spec fun update_valid_still_mutating {
        // sum should not change because we have not ended mutating t
        ensures sum_of_T == old(sum_of_T);
    }

    fun update_correct(): T {
        let t = T {i: 0};
        update_valid_still_mutating(&mut t);
        t
    }
    spec fun update_correct {
        // sum should change because we have ended mutating t
        ensures sum_of_T == old(sum_of_T) + 3;
    }

    fun update_incorrect(): T {
        let t = T {i: 0};
        update_valid_still_mutating(&mut t);
        t
    }
    spec fun update_incorrect {
        // sum should change because we have ended mutating t
        ensures sum_of_T == old(sum_of_T);
    }


    // -----------------------------------------------
    // Test with the combination of pack/unpack/update
    // -----------------------------------------------

    fun combi_correct() {
        let t = T{i:2};
        let r = T{i:3};
        let s = &mut r;
        s.i = 4;
        let T {i: _} = t;
    }
    spec fun combi_correct {
        ensures sum_of_T == old(sum_of_T) + 2 + 3 - 3 + 4 - 2;
    }

    fun combi_incorrect() {
        let t = T{i:2};
        let r = T{i:3};
        let s = &mut r;
        s.i = 4;
        let T {i: _} = t;
    }
    spec fun combi_incorrect {
        ensures sum_of_T == old(sum_of_T) + 2;
    }


    // ----------------------------------------------
    // Test with pack/unpack in the absence of update
    // ----------------------------------------------

    struct S {
      x: u64
    }
    spec struct S {
        global sum_of_S: u64;
        // If there are no update invariants, unpack and pack is used during mutation.
        invariant pack sum_of_S = sum_of_S + x;
        invariant unpack sum_of_S = sum_of_S - x;
    }

    fun update_S_correct(): S {
        let s = S {x: 0};
        let r = &mut s;
        r.x = 2;
        s
    }
    spec fun update_S_correct {
        ensures sum_of_S == old(sum_of_S) + 2;
    }

    fun update_S_incorrect(): S {
        let s = S {x: 0};
        let r = &mut s;
        r.x = 2;
        s
    }
    spec fun update_S_incorrect {
        ensures sum_of_S == old(sum_of_S) + 1;
    }
}
