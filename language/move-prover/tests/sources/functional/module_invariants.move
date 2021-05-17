address 0x1 {
module TestModuleInvariants {

    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed.

    spec module {
        pragma verify = true;
    }


    // Some structure.
    resource struct S {}

    // A resource tracking how many instances of S exist.
    resource struct SCounter {
        n: u64
    }

    // Resource invariants counting the number of S instances.
    spec S {
        // A counter for the # of alive instances of R
        global spec_count: u64;

        invariant pack spec_count = spec_count + 1;
        invariant unpack spec_count = spec_count - 1;
    }


    // A module invariant asserting that the resource SCounter correctly tracks what the specification expects.
    spec module {
        invariant global<SCounter>(0x0).n == spec_count;
    }

    // Creating and Deleting Resource
    // -------------------------------

    // Function creating an S instance. Since its public, we expect the module invariant to be satisfied.
    public fun new_S(): S acquires SCounter {
        let counter = borrow_global_mut<SCounter>(0x0);
        counter.n = counter.n + 1;
        S{}
    }

    // Function destroying an S instance. Since its public, we expect the module invariant to be satisfied.
    public fun delete_S(x: S) acquires SCounter {
        let S{} = x;
        let counter = borrow_global_mut<SCounter>(0x0);
        counter.n = counter.n - 1;
    }

    // Function destroying an S instance but not tracking it. The module invariant will catch this when the function
    // exits.
    public fun delete_S_incorrect(x: S) {
        let S{} = x;
    }

    // Private Calling Public
    // -----------------------

    // Private function calling a public function and failing because the pre-condition of the public function
    // does not hold.
    fun private_calls_public_incorrect(): S acquires SCounter {
       let x = new_S();
       x
    }

    // Private function calling a public function with correct precondition.
    fun private_calls_public(): S acquires SCounter {
        let x = new_S();
        x
    }
    spec private_calls_public {
        requires global<SCounter>(0x0).n == spec_count;
    }

*/
}


/*
module 0x42::TestModuleInvariantsExternal {
    use 0x1::TestModuleInvariants;

    public fun call_other() {
        // Module invariant satisfied because we call from other module.
        let x = TestModuleInvariants::new_S();
        TestModuleInvariants::delete_S(x);
    }
}
*/

}
