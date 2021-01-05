address 0x42 {
module M {
    struct S {
        f1: Option<u64>,
        f2: Option::Option<u64>,
    }

    fun bad_S(): S {
        S {
            f1: 0,
            f2: Option::none(),
        }
    }

    fun ex(
        _a1: Option<u64, u64>,
        s: &signer,
    ) {
        Option::none<u64>(0);
        Option::some();
        Signer::borrow_address(0x0);
        Signer::address_of(0x0);
        Vector::empty<u64>(0);
        Vector::push_back(0, 0);
    }
}
}
