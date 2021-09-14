module 0x42::UnusedGlobalInvariant {
    struct R0 has key {}
    struct R1<T: store> has key { t: T }
    struct R2 has key {}
    struct R3<T: store> has key { t: T }
    struct R4 has key {}

    fun publish_r1_bool(s: signer) {
        let r = R1 { t: true };
        move_to(&s, r);
    }

    fun publish_r2(s: signer) {
        let r = R2 {};
        move_to(&s, r);
    }
    spec publish_r2 {
        pragma delegate_invariants_to_caller;
    }

    fun publish_r3<T: store>(s: signer, t: T) {
        let r = R3 { t };
        move_to(&s, r);
    }
    spec publish_r3 {
        pragma delegate_invariants_to_caller;
    }
    fun call_publish_r3(s: signer) {
        publish_r3(s, true);
    }

    fun check_r4() {
        if (exists<R4>(@0x2)) {
            abort 42
        }
    }

    spec module {
        // This invariant is not checked anywhere in the code.
        // Because no function talks about R0
        invariant exists<R0>(@0x2) ==> exists<R0>(@0x3);

        // This invariant is not checked anywhere in the code.
        // Although publish_r1_bool modifies R1, it modifies R1<bool> and this
        // invariant is about R1<u64>
        invariant exists<R1<u64>>(@0x2) ==> exists<R1<u64>>(@0x3);

        // This invariant is not checked anywhere in the code.
        // Although publish_r2 modifies R2, it delegates the invariant to caller
        // and there is no caller to accept this invariant.
        invariant [suspendable] exists<R2>(@0x2) ==> exists<R2>(@0x3);

        // This invariant is not checked anywhere in the code.
        // Although publish_r3 modifies R3<T> it delegates the invariant to
        // caller, which is call_publish_r3, but call_publish_r3 modifies
        // R3<bool> and this invariant is about R3<u64>.
        invariant [suspendable] exists<R3<u64>>(@0x2) ==> exists<R3<u64>>(@0x3);

        // This invariant is not checked anywhere in the code.
        // Although check_r4 mentioned R4, there is no write operation that is
        // associated with R4. This invariant can be assumed in the beginning
        // of the function, but is never checked.
        invariant exists<R4>(@0x2) ==> exists<R4>(@0x3);
    }
}
