// flag: --v2
module TestFriend {
    resource struct R{
        x: u64,
    }


    public fun f(account: &signer, val: u64) {
        move_to(account, R{x: val});
    }
    spec fun f {
        pragma friend = gg;
    }

    spec fun f {
        /// This pragma declaration overwrites the previous one.
        pragma friend = g;
    }

    public fun g(account: &signer, val: u64) {
        f(account, val);
    }

    spec fun g {
        pragma friend = h;
    }

    public fun h(account: &signer) {
        g(account, 42);
    }

    spec module {
        /// Function f and g both violate this invariant on their own.
        /// However, since they can only be called from friend h's context, the following
        /// invariant can't be violated and the prover verifies with no errors.
        invariant [global] forall addr: address where exists<R>(addr): global<R>(addr).x == 42;
    }
}
