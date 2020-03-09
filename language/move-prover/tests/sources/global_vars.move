module GlobalVars {

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

    // Pack tests

    fun pack_valid(): T {
        T {i: 2}
    }
    spec fun pack_valid {
        ensures sum_of_T == old(sum_of_T) + 2;
    }

    fun pack_invalid(): T {
        T {i: 2}
    }
    spec fun pack_invalid {
        ensures sum_of_T == old(sum_of_T) + 1;
        ensures result.i == 2;
    }

    // Unpack tests

    fun unpack_valid(t: T): u64 {
        let T {i: x} = t;
        x
    }
    spec fun unpack_valid {
        ensures sum_of_T == old(sum_of_T) - old(t.i);
        ensures result == old(t.i);
    }

    fun unpack_invalid(t: T): u64 {
        let T {i: x} = t;
        x
    }
    spec fun unpack_invalid {
        ensures sum_of_T == old(sum_of_T);
        ensures result == old(t.i);
    }

    // Update tests.

    fun update_valid_still_mutating(t: &mut T) {
        t.i = t.i + 3;
    }
    spec fun update_valid_still_mutating {
        // sum should not change because we have not ended mutating t
        ensures sum_of_T == old(sum_of_T);
    }

    fun update_valid(): T {
        let t = T {i: 0};
        update_valid_still_mutating(&mut t);
        t
    }
    spec fun update_valid {
        // sum should change because we have ended mutating t
        ensures sum_of_T == old(sum_of_T) + 3;
    }

    fun update_invalid(): T {
        let t = T {i: 0};
        update_valid_still_mutating(&mut t);
        t
    }
    spec fun update_invalid {
        // sum should change because we have ended mutating t
        ensures sum_of_T == old(sum_of_T);
    }
}
