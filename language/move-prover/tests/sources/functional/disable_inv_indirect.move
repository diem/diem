module 0x1::M1 {
    //use Std::Signer;

    struct R has key { v: u64 }

    public fun f1(s: &signer) {
        move_to(s, R {v: 1});
    }

    public fun f2(s: &signer) {
        f1(s);
    }

    fun f_disabled(s: &signer) {
        f2(s);
    }
    spec f_disabled {
        pragma disable_invariants_in_body;
    }

    invariant [suspendable] forall addr: address: exists<R>(addr) ==> global<R>(addr).v > 0;
}
