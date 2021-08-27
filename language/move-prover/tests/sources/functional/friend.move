module 0x42::TestFriend {

    struct R has key {
        x: u64,
    }


    fun f(account: &signer, val: u64) {
        move_to(account, R{x: val});
    }
    spec f {
        pragma delegate_invariants_to_caller;
    }

    fun g(account: &signer, val: u64) {
        f(account, val);
    }
    spec g {
        pragma delegate_invariants_to_caller;
    }

    public fun h(account: &signer) {
        g(account, 42);
    }

    spec module {
        /// Function f and g both violate this invariant on their own.
        /// However, since they can only be called from h's context, the following
        /// invariant can't be violated and the prover verifies with no errors.
        invariant [suspendable] forall addr: address where exists<R>(addr): global<R>(addr).x == 42;
    }
}
