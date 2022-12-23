// separate_baseline: no_opaque
address 0x1 {

module M1 {
    use Std::Signer;
    use 0x1::M2;
    use 0x1::M3;

    public fun f1(s: &signer) {
        M2::f2(s);
        M3::f3(s);
    }

    spec f1 {
        pragma opaque;
        modifies global<M2::R2>(Signer::address_of(s));
        modifies global<M3::R3>(Signer::address_of(s));
    }

    spec f1 {
        pragma disable_invariants_in_body;
    }

    spec module {
         invariant [global, suspendable] forall addr: address: exists<M3::R3>(addr) <==> exists<M2::R2>(addr);
    }

}

module M2 {
    use Std::Signer;
    friend 0x1::M1;
    friend 0x1::M4;

    struct R2 has key {}

    public (friend) fun f2(s: &signer) {
        move_to(s, R2 {});
    }

    spec f2 {
        pragma opaque;
        modifies global<R2>(Signer::address_of(s));
        ensures exists<R2>(Signer::address_of(s));
    }
}

module M3 {
    use Std::Signer;
    friend 0x1::M1;
    friend 0x1::M4;

    struct R3 has key {}

    public(friend) fun f3(s: &signer) {
        move_to(s, R3 {});
    }

    spec f3 {
        pragma opaque;
        modifies global<R3>(Signer::address_of(s));
        ensures exists<R3>(Signer::address_of(s));
    }
}

module M4 {
    use Std::Signer;
    use 0x1::M2;
    use 0x1::M3;

    public fun f4(s: &signer) {
        M3::f3(s);
        M2::f2(s);
    }
    spec f4 {
        pragma opaque;
        modifies global<M2::R2>(Signer::address_of(s));
        modifies global<M3::R3>(Signer::address_of(s));
    }

    spec f4 {
        pragma disable_invariants_in_body;
    }

    public fun f5_incorrect(s: &signer) {
        M3::f3(s);
        M2::f2(s);
    }
    spec f5_incorrect {
        pragma opaque;
        modifies global<M2::R2>(Signer::address_of(s));
        modifies global<M3::R3>(Signer::address_of(s));
    }


}
}
