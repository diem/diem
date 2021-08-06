module 0x1::DisableInv {

    struct R1 has key { }

    struct R2 has key { }

    struct R3 has key { }

    // Error here because the function is public, it modifies the
    // invariant, and has a pragma disable_invariants_in_body
    public fun f1_incorrect(s: &signer) {
        move_to(s, R1 {});
        move_to(s, R2 {});
    }

    spec f1_incorrect {
         pragma delegate_invariants_to_caller;
    }

    public fun f2(s: &signer) {
        f3_incorrect(s);
        f4(s);
    }

    spec f2 {
        pragma disable_invariants_in_body;
    }

    // Error because it is called from a site (f2) where invariants
    // are disabled, but it has a pragma to disable invariants directly.
    fun f3_incorrect(s: &signer) {
        move_to(s, R1 {});
    }

    spec f3_incorrect {
        pragma disable_invariants_in_body;
    }

    fun f4(s: &signer) {
        f5_incorrect(s);
    }

    // Error because it is called from a site (f2) where invariants
    // are disabled, but it has a pragma to disable invariants directly.
    // Different from f3 because it's called indirectly through f4.
    fun f5_incorrect(s: &signer) {
        move_to(s, R2 {});
    }

    spec f5_incorrect {
        pragma disable_invariants_in_body;
    }

    // Lik f1_incorrect, but ok because it does not modify the invariant.
    public fun f6(s: &signer) {
        move_to(s, R3 {});
    }

    spec module {
        invariant [global] forall a: address where exists<R1>(a): exists<R2>(a);
    }
}
