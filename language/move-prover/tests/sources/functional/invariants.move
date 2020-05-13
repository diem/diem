module TestInvariants {

    spec module {
        pragma verify = true;
    }


    // General invariant checking.

    struct R {
        x: u64
    }

    spec struct R {
        // We must always have a value greater one.
        invariant greater_one(x);

        // When we update via a reference, the new value must be smaller or equal the old one.
        // invariant update x <= old(x) && tautology();
    }

    spec module {
        // Pure function to be used in data invariants.
        define greater_one(x: num): bool { x > 1 }

        // Impure function to be used in update invariants.
        define tautology() : bool { sender() == 0x0 || sender() != 0x0 }
    }


    // ----------
    // Pack tests
    // ----------

    fun valid_R_pack(): R {
        R {x: 2}
    }
    spec fun valid_R_pack {
        ensures result.x == 2;
    }

    fun invalid_R_pack(): R {
        R {x: 1}
    }
    spec fun invalid_R_pack {
        ensures result.x == 1;
    }


    // ------------
    // Update tests
    // ------------

    fun valid_R_update(): R {
        let t = R {x: 3};
        let r = &mut t;
        *r = R {x: 2};
        t
    }
    spec fun valid_R_update {
        ensures result.x == 2;
    }

    fun invalid_R_update(): R {
        let t = R {x: 3};
        let r = &mut t;
        *r = R {x: 4};
        t
    }
    spec fun invalid_R_update {
        ensures result.x == 4;
    }

    fun invalid_R_update_ref(): R {
        let t = R{x:3};
        let r = &mut t.x;
        *r = 4;
        t
    }
    spec fun invalid_R_update_ref {
        ensures result.x == 4;
    }

    fun invalid_R_update_indirectly(): R {
        let t = R{x:3};
        update_helper(&mut t.x);
        t
    }
    fun update_helper(r: &mut u64) {
        *r = 4;
    }

    fun invalid_R_update_branching(b: bool): R {
        let t1 = R {x: 5};
        let t2 = R {x: 3};
        let r: &mut R;
        if (b) {
            // this branch is fine because we can go from x = 5 to x = 4
            r = &mut t1
        } else {
            // this branch leads to update invariant violation as we cannot go from x = 3 to x = 4
            r = &mut t2
        };
        *r = R {x: 4};
        *r
    }

    // -----------------------
    // Lifetime analysis tests
    // -----------------------

    fun lifetime_invalid_R() : R {
        let r = R {x: 3};
        let r_ref = &mut r;
        let x_ref = &mut r_ref.x;
        *x_ref = 0; // r_ref goes out of scope here

        r_ref = &mut r;
        x_ref = &mut r_ref.x;
        *x_ref = 2;

        r
    }

    fun lifetime_invalid_R_2() : R {
        let r = R {x: 4};
        let r_ref = &mut r;
        let x_ref = &mut r_ref.x;
        *x_ref = 0;
        *x_ref = 2; // r_ref goes out of scope here

        r_ref = &mut r;
        x_ref = &mut r_ref.x;
        *x_ref = 3;

        r
    }

    struct T {
        x: u64
    }
    spec struct T {
        invariant x > 1;
    }

    struct S {
        y: u64
    }
    spec struct S {
        invariant y > 1;
    }

    fun lifetime_invalid_S_branching(cond: bool): (T, S) {
      let a = T {x: 3};
      let b = S {y: 4};
      let a_ref = &mut a;
      let b_ref = &mut b;
      let x_ref = if (cond) { &mut a_ref.x } else { &mut b_ref.y };

      if (cond) {
          *x_ref = 2;
      } else {
          *x_ref = 0;  // only S's invariant should fail
      };

      (a, b)
    }
}
