module TestModuleInvariants {

    // Some structure.
    struct S {}

    // A resource tracking how many instances of S exist.
    resource struct SCounter {
        n: u64
    }

    // Resource invariants counting the number of S instances.
    spec struct S {
        // A counter for the # of alive instances of R
        global spec_count: u64;

        invariant pack spec_count = spec_count + 1;
        invariant unpack spec_count = spec_count - 1;
    }


    // A module invariant asserting that the resource SCounter correctly tracks what the specification expects.
    spec module {
        invariant global<SCounter>(0x0).n == spec_count;
    }

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

    // Function destroying an S instance but not tracking it.
    public fun delete_S_invalid(x: S) {
        let S{} = x;
    }

    // Private function calling a public function and (currently) failing because we can't enforce the pre-condition
    // of the public function.
    fun private_calls_public_invalid(): S acquires SCounter {
       let x = new_S();
       x
    }
}
